use serde::{
    Deserialize,
    Serialize,
};

use std::fmt;

#[derive(Debug, Serialize, Deserialize)]
pub enum ProtocolError {
    InvalidMessage,
    MessageError,
    UserJoinedError,
    UserLeftError,
    AuthenticateError(authenticate::Error),
}

impl fmt::Display for ProtocolError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProtocolError::InvalidMessage => write!(f, "Mensagem inválida"),
            ProtocolError::MessageError => write!(f, "Erro ao tentar enviar mensagem"),
            ProtocolError::UserJoinedError => write!(f, "Erro ao tentar ao adicionar usuário ao chat"),
            ProtocolError::UserLeftError => write!(f, "Erro ao tentar remover usuário do chat"),
            ProtocolError::AuthenticateError(e) => write!(f, "Erro de autenticação: {e}"),
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum ClientProtocol {
    #[serde(rename = "send_message")]
    SendMessage { text: String },

    #[serde(rename = "join_chat")]
    JoinChat { username: String },

    #[serde(rename = "request_authenticate")]
    RequestAuthenticate { username: String, password: String },

    #[serde(rename = "add_user")]
    AddUser { username: String, password: String },

    /* 
    Protocols a implementar:
    RequestFeed,
    */
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ServerProtocol {
    #[serde(rename = "message")]
    Message { username: String, text: String },

    #[serde(rename = "user_joined")]
    UserJoined { username: String },

    #[serde(rename = "user_left")]
    UserLeft { username: String },

    #[serde(rename = "error")]
    Error { error: ProtocolError },

    #[serde(rename = "authenticated")]
    Authenticated,

    #[serde(rename = "user_added")]
    UserAdded,

    /* 
    Protocols a implementar:
    Feed,
    */
}
