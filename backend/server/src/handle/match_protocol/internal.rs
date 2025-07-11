use protocols::{
    ServerProtocol,
    Protocol,
};

use users::{
    Users,
    User,
};

use types::ArcUsers;

use crate::handle::match_protocol::utils::*;

pub async fn offline_message
(
    username: String,
    users: ArcUsers,
)
{
    let messages = Users::get_stored_messages(&username)
        .await;

    if messages.is_err() {
        eprintln!("Erro ao tentar recuperar as mensagens armazenadas");
        return;
    }

    let messages = messages.unwrap();

    if messages.len() == 0 {
        return;
    }

    println!("Há mensagens para você");

    let users = users.lock().await;

    // Essa função será chamada sempre que
    // um user ficar online, portanto é garantido
    // que get_user vai retornar um Some()
    let tx = users.get_user(User::new(&username)).await.unwrap();

    drop(users);

    for (sender, message) in messages {
        let reply = ServerProtocol::Message {
            from: sender,
            to: username.clone(),
            text: message,
        };

        let result = reply.serialize_and(async |json| {
            try_send(tx.clone(), &json).await;
            ServerProtocol::Success
        }).await;

        handle_result(tx.clone(), result).await;
    }

    let result = Users::delete_stored_messages(&username.clone())
        .await;

    if result.is_err() {
        eprintln!("Erro ao tentar excluir mensagens armazenadas");
    }
}
