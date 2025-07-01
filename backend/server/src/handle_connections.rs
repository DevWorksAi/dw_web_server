use std::{
    net::SocketAddr,
    sync::Arc,
};

use tokio::{
    sync::{mpsc::{
            UnboundedSender, 
            UnboundedReceiver,
            unbounded_channel,
        }, 
        Mutex
    },
};

use axum::{
    extract::{
        ws::{WebSocketUpgrade, WebSocket, Message},
        ConnectInfo,
        State,
    },
    response::Response,
};

use futures_util::{
    sink::SinkExt,
    stream::{StreamExt, SplitSink},
};

use error::{ProtocolError};

use protocols::{
    ClientProtocol, 
    ServerProtocol,
};

use users::{
    Users,
    User,
};

type Writer = Arc<Mutex<SplitSink<WebSocket, Message>>>;
type Tx = UnboundedSender<Message>;
type Rx = UnboundedReceiver<Message>;

pub async fn handler
(
    ws: WebSocketUpgrade,
    State(users): State<Users>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
) -> Response 
{
    // se o HTTPS vier com a tag UPGRADE, aprimora
    // o WebSocketUpgrade em um WebSocket
    ws.on_upgrade(move |socket| handle_socket(socket, users, addr))
}

pub async fn handle_socket
(
    socket: WebSocket,
    users: Users,
    addr: SocketAddr
)
{
    println!("client conectado: {addr}");
    // adiciona um novo consumidor
    let (tx, mut rx): (Tx, Rx) = unbounded_channel();
    let (write, mut read) = socket.split();

    let user = Arc::new(Mutex::new(User::new("")));
    let users = Arc::new(Mutex::new(users));

    // divide a websocket de forma mutavel em sua parte que le
    // e a que escreve

    let arc_write = Arc::new(Mutex::new(write));

    // task responsável por enviar ao broadcast
    let tx_task = {
        let user = Arc::clone(&user);
        let users = Arc::clone(&users);
        let write = Arc::clone(&arc_write);
        tokio::spawn(async move {
            // lê do websocket
            while let Some(Ok(msg)) = read.next().await {
                if let Message::Text(text) = msg {
                    match serde_json::from_str::<ClientProtocol>(&text) {
                        Ok(protocol) => handle_protocol(
                                            protocol, 
                                            Arc::clone(&user),
                                            Arc::clone(&users),
                                            tx.clone(),
                                            Arc::clone(&write),
                                        ).await,
                        Err(_) => {
                            let err = ServerProtocol::Error {
                                error: ProtocolError::InvalidMessage,
                            };

                            if let Ok(json) = serde_json::to_string(&err) {
                                let mut writer = write.lock().await;
                                let _ = writer.send(json.into());
                            } else {
                                eprintln!("Erro ao serializar ServerProtocol::Error");
                            }
                        }
                    } 
                }
            }
        })
    };

    // task responsável por ler do broadcast
    let rx_task = {
        let write = Arc::clone(&arc_write);

        tokio::spawn(async move {
                while let Some(msg) = rx.recv().await {
                    let mut writer = write.lock().await;
                    // devolve o que recebe do broadcast pra
                    // websocket
                    if writer.send(msg).await.is_err() {
                        break;
                    }
                }
            })
    };

    // espera até uma das duas tasks terminar primeiro
    tokio::select! {
        _ = tx_task => {}
        _ = rx_task => {}
    }

    // envia pro broadcast um userdisconnected
    let user = user.lock().await;
    let left_msg = ServerProtocol::UserDisconnected { username: user.username.clone() };

    if let Ok(json) = serde_json::to_string(&left_msg) {
        let users = users.lock().await;
        let on_users = users.on_users.lock().await;

        if let Some(tx) = on_users.get(&user) {
            let _ = tx.send(json.into());
        } else {
            panic!("O programa tentou desconectar um usuário que não existe. Por favor, garanta que o usuário esteja autenticado antes de fazer qualquer teste enviando um RequestAuthenticate");
        }
    } else {
        eprintln!("Erro ao serializar ServerProtocol::UserDisconnected");
    }
}

async fn handle_protocol
(
    protocol: ClientProtocol,
    user: Arc<Mutex<User>>,
    users: Arc<Mutex<Users>>,
    tx: Tx,
    write: Writer,
)
{
    match protocol {
        ClientProtocol::SendMessage { from, to, text } => {
            // se alguem pode enviar mensagem significa
            // que ele já está autenticado, e portanto
            // está no hashmap de users
            // agora se "to" não está no hash, entao
            // um erro é retornado

            let reply = ServerProtocol::Message {
                from,
                to: to.clone(),
                text,
            };

            if let Ok(json) = serde_json::to_string(&reply) {
                let users = users.lock().await;
                let on_users = users.on_users.lock().await;

                if let Some(target) = on_users.get(&User::new(&to)) {
                    let _ = target.send(json.into());
                } else {
                    let err = ServerProtocol::Error {
                        error: ProtocolError::UserNotExist,
                    };

                    if let Ok(json) = serde_json::to_string(&err) {
                        let mut writer = write.lock().await;
                        let _ = writer.send(Message::Text(json.into())).await;
                        println!("oi");
                    } else {
                        eprintln!("Erro ao serializar ServerProtocol::UserNotExist");
                    }
                }
            } else {
                eprintln!("Erro ao serializar ServerProtocol::Message");
            }
        },

        ClientProtocol::RequestAuthenticate { username, password } => {
            *user.lock().await = User::new(&username);
            let mut users = users.lock().await;

            match users.authenticate_user(&username, &password, tx).await {
                Ok(()) => {
                    let mut writer = write.lock().await;
                    let authenticated = ServerProtocol::Authenticated;

                    if let Ok(json) = serde_json::to_string(&authenticated) {
                        let _ = writer.send(Message::Text(json.into())).await;
                    } else {
                        eprintln!("Erro ao serializar ServerProtocol::Authenticated");
                    }
                },
                Err(e) => {
                    let mut writer = write.lock().await;

                    let err = ServerProtocol::Error {
                        error: ProtocolError::AuthenticateError(e),
                    };

                    if let Ok(json) = serde_json::to_string(&err) {
                        let _ = writer.send(Message::Text(json.into())).await;
                    } else {
                        eprintln!("Erro ao serializar ServerProtocol::Error");
                    }
                }

            }
        },

        ClientProtocol::CreateUser {username, password} => {
            match Users::add_user(&username, &password).await {
                Ok(()) => {
                    let mut writer = write.lock().await;

                    let added = ServerProtocol::UserCreated;

                    if let Ok(json) = serde_json::to_string(&added) {
                        let _ = writer.send(Message::Text(json.into())).await;
                    } else {
                        eprintln!("Error ao serializar ServerProtocol::UserAdded");
                    }
                },

                Err(e) => {
                   let mut writer = write.lock().await;
                   
                   let err = ServerProtocol::Error {
                        error: ProtocolError::AuthenticateError(e),
                   };

                   if let Ok(json) = serde_json::to_string(&err) {
                        let _ = writer.send(Message::Text(json.into())).await;
                   } else {
                        eprintln!("Erro ao tentar serializar ServerProtocol::Error");
                   }
                }
            }
        },
    }
}

