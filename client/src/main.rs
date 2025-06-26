#![allow(unused_imports, unused_variables, unused_mut)]
    
use futures_util::{
    sink::SinkExt,
    stream::StreamExt,
};

use tokio_tungstenite::{
    connect_async,
    WebSocketStream,
    MaybeTlsStream,
};

use tungstenite::{
    http::{Method, Request},
    client::IntoClientRequest,
    Message,
};

use tokio::{
    io::{self, AsyncBufReadExt, BufReader},
    net::TcpStream,
};

#[tokio::main]
async fn main() {
    let mut request = "ws://localhost:3000/ws".
    into_client_request().unwrap();

    let (mut socket, response) = connect_async(request).
        await.unwrap();

    let (mut write, mut read) = socket.split();

    println!("conectastesse ao melhr chat of the world seloko (CTRL + D para sair)");

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

    let reader = tokio::spawn(async move {
        while let Some(Ok(msg)) = read.next().await {
            if let Message::Text(text) = msg {
                println!("mensagem recebida: {}", text);
            }
        }
    });

    let _ = tokio::join!(sender, reader);
}
