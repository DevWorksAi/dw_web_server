use std::{
    net::SocketAddr,
    sync::Arc,
};

use tokio::{
    sync::{broadcast, Mutex},
    io::Result,
};

use axum::{
    Router,
    routing::any,
    extract::{
        ws::{WebSocketUpgrade, WebSocket, Message},
        ConnectInfo,
    },
    response::Response,
    Extension,
};

use futures_util::{
    sink::SinkExt,
    stream::StreamExt,
};

use protocols::{
    ClientProtocol, 
    ServerProtocol,
};

type Tx = broadcast::Sender<String>;

async fn handler
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

async fn handle_socket
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
    let (mut write, mut read) = socket.split();

    // task responsável por enviar ao broadcast
    let tx_task = {
        let tx_clone = tx.clone();
        let username = Arc::clone(&username);

        tokio::spawn(async move {
            // lê do websocket
            while let Some(Ok(msg)) = read.next().await {
                if let Message::Text(text) = msg {
                    // transforma o json em ClientProtocol
                    match serde_json::from_str::<ClientProtocol>(&text) {
                        Ok(ClientProtocol::JoinChat { username: name }) => {
                            *username.lock().await = name.clone();

                            let joined = ServerProtocol
                            ::UserJoined { username: name };

                            // envia pro broadcast
                            let _ = tx_clone.send(serde_json
                                ::to_string(&joined).unwrap());
                        },

                        Ok(ClientProtocol::SendMessage { text }) => {
                            let name = username.lock().await.clone();

                            let reply = ServerProtocol::Message {
                                username: name,
                                text,
                            };

                            // envia pro broadcast
                            let _ = tx_clone.send(serde_json
                                ::to_string(&reply).unwrap());
                        },

                        Err(_) => {
                            let err = ServerProtocol::Error {
                                message: "mensagem inválida".into(),
                            };

                            // envia pro broadcast
                            let _ = tx_clone.send(serde_json
                                ::to_string(&err).unwrap());
                        },
                    }
                }
            }
        })
    };

    // task responsável por ler do broadcast
    let rx_task = tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            // devolve o que recebe do broadcast pra
            // websocket
            if write.send(Message::Text(msg.into())).await.is_err() {
                break;
            }
        }
    });

    // espera até uma das duas tasks terminar primeiro
    tokio::select! {
        _ = tx_task => {}
        _ = rx_task => {}
    }

    let name = username.lock().await.clone();

    // envia pro broadcast um userleft
    let left_msg = ServerProtocol::UserLeft { username: name };
    let _ = tx.send(serde_json::to_string(&left_msg).unwrap());
}


#[tokio::main]
async fn main() -> Result<()> {
    // cria o broadcast (single sender, multiple consumer)
    let (tx, _) = broadcast::channel::<String>(16);

    // cria a estrutura do server
    let app = Router::<()>::new()
        .route("/ws", any(handler))
        .layer(Extension(tx));

    // ouve via tcp no endereço dado
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    let listener = tokio::net::TcpListener::bind(addr)
        .await?;

    println!("Server rodando em ws::/{addr}");

    // cria efetivamente o servidor web
    axum::serve(listener,
        app.into_make_service_with_connect_info::<SocketAddr>())
        .await?;

    Ok(())
}