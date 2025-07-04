/*
Aqui estão implementados os tipos de erro
para o tratamento e propagação de erros durante 
o programa inteiro se tornar mais idiomático.
*/

use serde::{
    Deserialize,
    Serialize,
};

use std::{
    fmt,
};

// Erros relacionados a database mysql e usuários.
#[derive(Debug, Serialize, Deserialize)]
pub enum AuthenticateErrorType {
    Std,
    Hash,
    Sql,
    Envy,
    PasswordMismatch,
    UserNotFound,
    UserNotAdded,
    UserAlreadyExists,
}

impl From<argon2::password_hash::Error> for AuthenticateErrorType {
    fn from(_: argon2::password_hash::Error) -> Self {
        Self::Hash
    }
}

impl From<sqlx::Error> for AuthenticateErrorType {
    fn from(_: sqlx::Error) -> Self {
        Self::Sql
    }
}

impl From<dotenvy::Error> for AuthenticateErrorType {
    fn from(_: dotenvy::Error) -> Self {
        Self::Envy
    }
}

impl std::error::Error for AuthenticateErrorType {}

impl fmt::Display for AuthenticateErrorType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AuthenticateErrorType::Std => write!(f, "Erro de std"),
            AuthenticateErrorType::Hash => write!(f, "Erro de argon2"),
            AuthenticateErrorType::Envy => write!(f, "Erro de dotenvy"),
            AuthenticateErrorType::Sql => write!(f, "Erro de slqx"),
            AuthenticateErrorType::PasswordMismatch => write!(f, "Senha inválida"),
            AuthenticateErrorType::UserNotFound => write!(f, "Usuário não encontrado"),
            AuthenticateErrorType::UserNotAdded => write!(f, "Usuário não foi cadastrado"),
            AuthenticateErrorType::UserAlreadyExists => write!(f, "Usuário já está cadastrado"),
        }
    }
}

// Erro genérico que contêm todos
// os erros possíveis usados nesse programa.
#[derive(Debug, Serialize, Deserialize)]
pub enum ProtocolError {
    Serde,
    InvalidMessage,
    MessageError,
    UserJoinedError,
    UserDisconnectedError,
    UserNotExist,
    UserOffline,
    AuthenticateError(AuthenticateErrorType),
}


impl From<serde_json::Error> for ProtocolError {
    fn from(_: serde_json::Error) -> Self {
        Self::Serde
    }
}


impl fmt::Display for ProtocolError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProtocolError::InvalidMessage => write!(f, "Mensagem inválida"),
            ProtocolError::MessageError => write!(f, "Erro ao tentar enviar mensagem"),
            ProtocolError::UserJoinedError => write!(f, "Erro ao tentar ao adicionar usuário ao chat"),
            ProtocolError::UserDisconnectedError => write!(f, "Erro ao tentar remover usuário do chat"),
            ProtocolError::UserNotExist => write!(f, "Usuário inexistente"),
            ProtocolError::UserOffline => write!(f, "Usuário offline"),
            ProtocolError::AuthenticateError(e) => write!(f, "Erro de autenticação: {e}"),
            ProtocolError::Serde => write!(f, "Erro ao tentar serializar/deserializar uma mensagem"),
        }
    }
}
