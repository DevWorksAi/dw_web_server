use error::{ProtocolError};

use protocols::{
    ServerProtocol,
    InternalProtocol,
    Protocol,
};

use users::{
    Users,
    User,
};

use types::{Tx, TxInt, ArcUser, ArcUsers};

use crate::handle::match_protocol::utils::*;

pub async fn send_message
(
    from: String,
    to: String,
    text: String,
    users: ArcUsers,
    tx: Tx,
)
{
    // Se alguém pode enviar mensagem significa
    // que ele já está autenticado e, portanto
    // está no hashmap de users. (é obrigação
    // do client garantir que o usuário esteja autenticado).
    // Agora se "to" não está no hash, entao
    // um erro é retornado (por enquanto).

    let reply = ServerProtocol::Message {
        from: from.clone(),
        to: to.clone(),
        text: text.clone(),
    };

    let result = reply.serialize_and(async |json| {
        let users = users.lock().await;

        if let Some(target) = users.get_user(User::new(&to))
        .await 
        {
            drop(users);

            try_send(target.clone(), &json).await;
            ServerProtocol::Success
        } else {
            drop(users);

            match Users::user_exists(&to).await {
                Ok(result) => {
                    if !result {
                        let err = ServerProtocol::Error {
                            error: ProtocolError::UserNotExist,
                        };

                        handle_instance(tx.clone(), err).await;
                        return ServerProtocol::Success
                    }
                },
                Err(e) => {
                    let err = ServerProtocol::Error {
                        error: ProtocolError::AuthenticateError(e),
                    };

                    handle_instance(tx.clone(), err).await;
                    return ServerProtocol::Success
                }
            }

            let result = Users
            ::store_message(&from, &to, &text)
            .await;

            if result.is_err() {
                eprintln!("Erro ao tentar armazenar mensagens");
            }

            ServerProtocol::Success
        }
    }).await;

    handle_result(tx, result).await;
}

pub async fn request_authenticate
(
    username: String,
    password: String,
    user: ArcUser,
    users: ArcUsers,
    tx: Tx,
    txi: TxInt,
)
{
    *user.lock().await = User::new(&username);
    let mut users = users.lock().await;

    match users.authenticate_user(&username, &password, tx.clone()).await {
        Ok(()) => {
            drop(users);

            let authenticated = ServerProtocol::Authenticated;
            handle_instance(tx.clone(), authenticated).await;

            // Envia um sinal para verificar as possíveis
            // mensagens que foram armazenadas enquanto
            // o usuário esteve offline.
            let check_stored_messages = InternalProtocol::OfflineMessage {
                username: username,
            };

            if txi.send(check_stored_messages).is_err() {
                eprintln!(
                "Erro ao tentar enviar pelo channel; Motivo: rxi foi dropado");
            }
        },
        Err(e) => {
            drop(users);
            let err = ServerProtocol::Error {
                error: ProtocolError::AuthenticateError(e),
            };

            handle_instance(tx, err).await;
        }
    }
}

pub async fn create_user
(
    username: String,
    password: String,
    tx: Tx,
)
{
    match Users::add_user(&username, &password).await {
        Ok(()) => {
            let added = ServerProtocol::UserCreated;

            handle_instance(tx, added).await;
        },

        Err(e) => {
            let err = ServerProtocol::Error {
                error: ProtocolError::AuthenticateError(e),
            };

            handle_instance(tx, err).await;
        }
    }    
}
