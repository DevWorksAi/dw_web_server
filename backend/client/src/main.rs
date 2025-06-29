use futures_util::{
    sink::SinkExt,
    stream::StreamExt,
};

use tokio_tungstenite::{
    connect_async,
};

use tungstenite::{
    client::IntoClientRequest,
    Message,
};

use tokio::{
    io::{self, AsyncBufReadExt, BufReader},
};

use tokio_util::{
    sync::CancellationToken,
};

use protocols::{ServerProtocol, ClientProtocol};

#[tokio::main]
async fn main() {
    let request = "ws://localhost:3000/ws".
    into_client_request().unwrap();

    let (socket, _response) = connect_async(request).
        await.unwrap();

    let (mut write, mut read) = socket.split();

    let (username, password) = (String::from("Artur"), String::from("1234"));

    // apague o comentario para tentar adicionar algum usuario,
    // basta mudar (username, password) ali em cima
    // write
    //     .send(Message::Text(
    //         serde_json::to_string(&ClientProtocol::AddUser {
    //             username: username.clone(),
    //             password: password.clone(),
    //         }).unwrap().into()
    //     ))
    //     .await
    //     .expect("Erro ao enviar mensagem");


    write
        .send(Message::Text(
            serde_json::to_string(&ClientProtocol::RequestAuthenticate {
                username: username.clone(),
                password: password.clone(),
            }).unwrap().into()
        ))
        .await
        .expect("Erro ao enviar mensagem");

    write.
        send(Message::Text(
            serde_json::to_string(&ClientProtocol::JoinChat {
                username,
            }).unwrap().into()
        ))
        .await
        .unwrap();

    println!("conectastesse ao melhr chat of the world seloko (CTRL + D para sair)");

    let cancel = CancellationToken::new();
    let cancel_sender = cancel.clone();

    let sender = {
        tokio::spawn(async move {
            let stdin = BufReader::new(io::stdin());
            let mut lines = stdin.lines();

            loop {
                tokio::select! {
                    _ = cancel_sender.cancelled() => {
                        println!("cancelado");
                        break;
                    }

                    line = lines.next_line() => {
                        if let Ok(Some(line)) = line {
                            if line.is_empty() {
                                continue;
                            }

                            let msg = ClientProtocol::SendMessage { text: line };
                            let json = serde_json::to_string(&msg).unwrap();

                            write
                                .send(Message::Text(json.into()))
                                .await
                                .expect("erro ao enviar mensagem");                        
                        } else {
                            break;
                        }
                    }
                }
            }
        })
    };

    let reader = tokio::spawn(async move {
        while let Some(Ok(msg)) = read.next().await {
            if let Message::Text(json) = msg {
                match serde_json::from_str::<ServerProtocol>(&json) {
                    Ok(ServerProtocol::Message { username, text }) => {
                        println!("{username}: {text}");
                    },

                    Ok(ServerProtocol::UserJoined { username }) => {
                        println!("{username} entrou na conversa!");
                    },

                    Ok(ServerProtocol::UserLeft { username }) => {
                        println!("{username} saiu da conversa");
                    },

                    Ok(ServerProtocol::Authenticated) => {
                        println!("Usuário autenticado com sucesso");
                    },

                    Ok(ServerProtocol::UserAdded) => {
                        println!("Usuário adicionado no banco de dados");
                    },

                    Ok(ServerProtocol::Error { error }) => {
                        println!("Erro -> {error}");
                        break;
                    },

                    Err(_) => {
                        println!("algum problema nada poggers aconteceukkk");
                    },
                }
            }
        }
    });

    tokio::select! {
        _ = sender => {}
        _ = reader => { cancel.cancel(); }
    }
}
