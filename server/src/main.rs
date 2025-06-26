#![allow(unused_imports, unused_variables, unused_mut)]

use std::{
    net::SocketAddr,
};

use tokio::{
    sync::broadcast,
    io::{self, AsyncBufReadExt, BufReader, Result},
};

use axum::{
    Router,
    routing::any,
    extract::{
        ws::{WebSocketUpgrade, WebSocket, Message},
        ConnectInfo,
    },
    response::{IntoResponse, Response},
};

use futures_util::{
    sink::SinkExt,
    stream::StreamExt,
};



async fn handler
(
    ws: WebSocketUpgrade,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
) -> Response 
{
    ws.on_upgrade(move |socket| handle_socket(socket, addr))
}

async fn handle_socket
(
    mut socket: WebSocket, 
    addr: SocketAddr
) 
{
    println!("client conectado: {addr}");


    let (mut write, mut read) = socket.split();

    let reader = tokio::spawn(async move{
        while let Some(Ok(msg)) = read.next().await {
            if let Message::Text(text) = msg {
                println!("mensagem recebida: {}", text);
            }
        }
    });

    let sender = tokio::spawn(async move {
        let stdin = BufReader::new(io::stdin());
        let mut lines = stdin.lines();

        while let Ok(Some(line)) = lines.next_line().await {
            if line.is_empty() {
                continue;
            }

            write
                .send(Message::Text(line.into()))
                .await
                .expect("erro ao enviar mensagem");
        }
    });


    let _ = tokio::join!(sender, reader);
}

#[tokio::main]
async fn main() -> Result<()> {
    let app = Router::<()>::new()
        .route("/ws", any(handler));


    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await?;

    axum::serve(listener,
        app.into_make_service_with_connect_info::<SocketAddr>())
        .await?;

    Ok(())
}