use std::{
    net::SocketAddr,
    sync::Arc,
};

use tokio::
    sync::{
        mpsc::unbounded_channel, 
        Mutex,
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
    stream::StreamExt,
};

use error::ProtocolError;


use protocols::{
    ClientProtocol, 
    ServerProtocol,
    Protocol,
};

use users::{
    Users,
    User,
};

use crate::handle::handle_protocols::handle_protocol;
use crate::handle::types::{Tx, Rx, ArcReader,
                           ArcWriter, ArcUser, ArcUsers};

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
    let (tx, rx): (Tx, Rx) = unbounded_channel();
    let (write, read) = socket.split();

    let reader = Arc::new(Mutex::new(read));
    let writer = Arc::new(Mutex::new(write));
    let user = Arc::new(Mutex::new(User::new("")));
    let users = Arc::new(Mutex::new(users));

    // task responsável por enviar ao broadcast
    let tx_task = tokio::spawn(receive_from_socket(
        Arc::clone(&reader),
        Arc::clone(&writer),
        Arc::clone(&user),
        Arc::clone(&users),
        tx.clone(),
    ));

    // task responsável por ler do broadcast
    let rx_task = tokio::spawn(send_to_socket(
        Arc::clone(&writer),
        rx,
    ));

    // espera até uma das duas tasks terminar primeiro
    tokio::select! {
        _ = tx_task => {}
        _ = rx_task => {}
    }
}

async fn receive_from_socket
(
    reader: ArcReader,
    writer: ArcWriter,
    user: ArcUser,
    users: ArcUsers,
    tx: Tx,
)
{  
    let mut reader = reader.lock().await;

    while let Some(Ok(msg)) = reader.next().await {
        if let Message::Text(text) = msg {
            match serde_json::from_str::<ClientProtocol>(&text) {
                Ok(protocol) => handle_protocol(
                                    protocol, 
                                    user.clone(),
                                    users.clone(),
                                    tx.clone(),
                                    writer.clone(),
                                ).await,
                Err(_) => {
                    let err = ServerProtocol::Error {
                        error: ProtocolError::InvalidMessage,
                    };

                    let _ = err.serialize_and(async |json| {
                        let mut writer = writer.lock().await;
                        let _ = writer.send(json.into());
                    }).await;
                }
            } 
        }
    }
}

async fn send_to_socket
(
    writer: ArcWriter,
    mut rx: Rx,
)
{
    while let Some(msg) = rx.recv().await {
        let mut writer = writer.lock().await;
        if writer.send(msg).await.is_err() {
            break;
        }
    }
}