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

use protocols::{
    ClientProtocol, 
    Protocol,
};

use users::{
    Users,
    User,
};

use crate::handle::handle_protocols::{
    handle_protocol,
    handle_internal,
};

use types::{Tx, Rx, ArcReader,
    ArcWriter, ArcUser, ArcUsers,
    TxInt, RxInt};

// Responsável por lidar com o Https recebido do client
pub async fn handler
(
    ws: WebSocketUpgrade,
    State(users): State<Users>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
) -> Response 
{
    // Se o HTTPS vier com a tag UPGRADE, aprimora
    // o WebSocketUpgrade em um WebSocket
    ws.on_upgrade(move |socket| handle_socket(socket, users, addr))
}

// Função principal para leitura e envio de 
// dados pela socket especificamente. Um channel
// também é criado, mas ele é tratado por outras
// funções que handle_socket chama.
pub async fn handle_socket
(
    socket: WebSocket,
    users: Users,
    addr: SocketAddr
)
{
    println!("client conectado: {addr}");
    // Cria a socket e o channel.
    let (tx, rx): (Tx, Rx) = unbounded_channel();
    let (txi, rxi): (TxInt, RxInt) = unbounded_channel();
    let (write, read) = socket.split();

    let reader = Arc::new(Mutex::new(read));
    let writer = Arc::new(Mutex::new(write));
    let user = Arc::new(Mutex::new(User::new("")));
    let users = Arc::new(Mutex::new(users));

    // Task responsável pela leitura
    let tx_task = tokio::spawn(receive_from_socket(
        Arc::clone(&reader),
        Arc::clone(&user),
        Arc::clone(&users),
        tx.clone(),
        txi.clone(),
    ));

    // Task responsável pelo canal interno
    let int_channel_task = tokio::spawn(handle_internal_channel(
        Arc::clone(&users),
        rxi,
    ));

    // Task responsável pelo envio
    let rx_task = tokio::spawn(send_to_socket(
        writer,
        rx,
        addr,
    ));

    // Espera até uma das duas tasks acima criadas
    // terem terminado, cancelando de forma segura
    // a outra.
    tokio::select! {
        _ = tx_task => {}
        _ = rx_task => {}
        _ = int_channel_task => {}
    }

    println!("client desconectado: {addr:?}");
    let mut users = users.lock().await;
    let user = user.lock().await;
    users.remove_user(&*user.username).await;
}

// Função responsável pela leitura de dados.
async fn receive_from_socket
(
    reader: ArcReader,
    user: ArcUser,
    users: ArcUsers,
    tx: Tx,
    txi: TxInt,
)
{  
    let mut reader = reader.lock().await;

    while let Some(Ok(msg)) = reader.next().await {
        if let Message::Text(text) = msg {
            let _ = ClientProtocol
            ::deserialize_and(&text, async |protocol| {
                handle_protocol(
                        protocol, 
                        user.clone(),
                        users.clone(),
                        tx.clone(),
                        txi.clone(),
                ).await;             
            }).await;
        }
    }
}

// Função responsável pelo envio de dados.
async fn send_to_socket
(
    writer: ArcWriter,
    mut rx: Rx,
    addr: SocketAddr,
)
{
    while let Some(msg) = rx.recv().await {
        let mut writer = writer.lock().await;
        if writer.send(msg).await.is_err() {
            let _ = writer.close().await;
            eprintln!("Conexão com {addr:?} foi fechada");
            break;
        }
    }
}

async fn handle_internal_channel
(
    users: ArcUsers,
    mut rx: RxInt,
)
{
    while let Some(msg) = rx.recv().await {
        handle_internal(
            msg,
            users.clone(),
        ).await;
    }
}