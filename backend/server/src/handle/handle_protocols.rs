use protocols::{
    ClientProtocol, 
    InternalProtocol,
};

use types::{Tx, TxInt, ArcUser, ArcUsers};

use crate::handle::match_protocol::client::{
    send_message,
    request_authenticate,
    create_user,
};

use crate::handle::match_protocol::internal::offline_message;

// Lida com ClientProtocol's enviados pelo client.
pub async fn handle_protocol
(
    protocol: ClientProtocol,
    user: ArcUser,
    users: ArcUsers,
    tx: Tx,
    txi: TxInt,
)
{   
    // Os drops explícitos são usadas para
    // liberar o Mutex o mais cedo possível, isto é,
    // na medida em que o Mutex não é mais necessário, para não
    // bloquear o valor por mais tempo que o necessário.
    match protocol {
        ClientProtocol::SendMessage { from, to, text } => {
            send_message(
                from,
                to,
                text,
                users,
                tx,
            ).await
        },

        ClientProtocol::RequestAuthenticate { username, password } => {
            request_authenticate(
                username,
                password,
                user,
                users,
                tx,
                txi,
            ).await
        },

        ClientProtocol::CreateUser {username, password} => {
            create_user(
                username,
                password,
                tx,
            ).await
        },
    }
}


pub async fn handle_internal
(
    protocol: InternalProtocol,
    users: ArcUsers,
)
{
    match protocol {
        InternalProtocol::OfflineMessage { username } => {
            offline_message(
                username,
                users,
            ).await
        }
    }
}

