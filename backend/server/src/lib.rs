use std::{
    net::SocketAddr,
    sync::Arc,
};

use tokio::{
    sync::{broadcast, Mutex},
};

use axum::{
    extract::{
        ws::{WebSocketUpgrade, WebSocket, Message},
        ConnectInfo,
    },
    response::Response,
    Extension,
};

use futures_util::{
    sink::SinkExt,
    stream::{StreamExt, SplitSink},
};

use protocols::{
    ClientProtocol, 
    ServerProtocol,
    ProtocolError,
};

use authenticate::{
    authenticate_user,
    connect_to_database,
    add_user,
};

type Tx = broadcast::Sender<String>;
type Writer = Arc<Mutex<SplitSink<WebSocket, Message>>>;

pub async fn handler
(
    ws: WebSocketUpgrade,
    Extension(tx): Extension<Tx>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
) -> Response 
{
    // se o HTTPS vier com a tag UPGRADE, aprimora
    // o WebSocketUpgrade em um WebSocket
    ws.on_upgrade(move |socket| handle_socket(socket, tx, addr))
}

async fn handle_protocol
(
    protocol: ClientProtocol,
    username: Arc<Mutex<String>>,
    tx: Tx,
    write: Writer,
)
{
    match protocol {
        ClientProtocol::JoinChat { username: name } => {
            *username.lock().await = name.clone();

            let joined = ServerProtocol
            ::UserJoined { username: name };

            if let Ok(json) = serde_json::to_string(&joined) {
                let _ = tx.send(json);
            } else {
                eprintln!("Erro ao serializar ServerProtocol::UserJoined");
            }
        },

        ClientProtocol::SendMessage { text } => {
            let name = username.lock().await.clone();

            let reply = ServerProtocol::Message {
                username: name,
                text,
            };

            if let Ok(json) = serde_json::to_string(&reply) {
                let _ = tx.send(json);
            } else {
                eprintln!("Erro ao serializar ServerProtocol::Message");
            }
        },

        ClientProtocol::RequestAuthenticate { username: name, password } => {
            *username.lock().await = name.clone();

            match connect_to_database().await {
                Ok(pool) => {
                    match authenticate_user(&pool, &name, &password).await {
                        Ok(()) => {
                            let mut writer = write.lock().await;

                            let authenticated = ServerProtocol::Authenticated;

                            if let Ok(json) = serde_json::to_string(&authenticated) {
                                let _ = writer.send(Message::Text(json.into())).await;
                            } else {
                                eprintln!("Erro ao serializar ServerProtocol::Authenticated");
                            }

                        }
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

        ClientProtocol::AddUser {username: name, password} => {
            match connect_to_database().await {
                Ok(pool) => {
                    match add_user(&pool, &name, &password).await {
                        Ok(()) => {
                            let mut writer = write.lock().await;

                            let added = ServerProtocol::UserAdded;

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
                                eprintln!("Erro ao serializar ServerProtocol::Error");
                            }                                            
                        }
                    }
                }
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

pub async fn handle_socket
(
    socket: WebSocket,
    tx: Tx,
    addr: SocketAddr
)
{
    println!("client conectado: {addr}");
    // adiciona um novo consumidor
    let mut rx = tx.subscribe();

    // necessario para username ser compatilhado entre tasks
    let username = Arc::new(Mutex::new(String::from("anônimo")));

    // divide a websocket de forma mutavel em sua parte que le
    // e a que escreve
    let (write, mut read) = socket.split();

    let arc_write = Arc::new(Mutex::new(write));

    // task responsável por enviar ao broadcast
    let tx_task = {
        let tx_clone = tx.clone();
        let username = Arc::clone(&username);
        let write = Arc::clone(&arc_write);
        tokio::spawn(async move {
            // lê do websocket
            while let Some(Ok(msg)) = read.next().await {
                if let Message::Text(text) = msg {
                    match serde_json::from_str::<ClientProtocol>(&text) {
                        Ok(protocol) => handle_protocol(
                                            protocol, 
                                            Arc::clone(&username),
                                            tx_clone.clone(),
                                            Arc::clone(&write),
                                        ).await,
                        Err(_) => {
                            let err = ServerProtocol::Error {
                                error: ProtocolError::InvalidMessage,
                            };

                            if let Ok(json) = serde_json::to_string(&err) {
                                let _ = tx_clone.send(json);
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
                while let Ok(msg) = rx.recv().await {
                    let mut writer = write.lock().await;
                    // devolve o que recebe do broadcast pra
                    // websocket
                    if writer.send(Message::Text(msg.into())).await.is_err() {
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

    let name = username.lock().await.clone();

    // envia pro broadcast um userleft
    let left_msg = ServerProtocol::UserLeft { username: name };

    if let Ok(json) = serde_json::to_string(&left_msg) {
        let _ = tx.send(json);
    } else {
        eprintln!("Erro ao serializar ServerProtocol::UserLeft");
    }
}
