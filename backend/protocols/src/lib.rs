/*
Protocols são os dados efetivamente enviados e recebidos
pela WebSocket que conecta server e client.
Esses dados são, em baixo nível, apenas arquivos json
formatados da forma correta.

Formatação de um protocol:

{
    "type": "nome_do_protocolo",
    ----- outras informações -----
}
*/

use serde::{
    Deserialize,
    Serialize,
};

use error::{
	ProtocolError,
};

use std::future::Future;


// Dá o poder de ServerProtocol e ClientProtocol serem
// deserializados e serializados de forma mais idiomática.
// A existência dessa trait é o motivo de ServerProtocol::Success
// existir.
pub trait Protocol: Sized + Serialize + for<'de> Deserialize<'de> {
    fn serialize_and<R, F, Fut>(self, f: F) -> 
        impl Future<Output = Result<R, ProtocolError>> + Send
    where
        Self: Serialize + Send + Sized + 'static,
        F: FnOnce(String) -> Fut + Send,
        Fut: Future<Output = R> + Send,
        R: Send,
    {
        async move {
            let string = serde_json::to_string(&self)?;
            let result = f(string).await;
            Ok(result)
        }
    }

    fn deserialize_and<R, F, Fut>(s: &str, f: F) -> 
        impl Future<Output = Result<R, ProtocolError>> + Send
    where
        F: FnOnce(Self) -> Fut + Send,
        Fut: Future<Output = R> + Send,
        R: Send,
    {   
        async move {
            let val = serde_json::from_str(s)?;
            let result = f(val).await;
            Ok(result)
        }
    }
}

// Protocolos enviados pelo client ao server
// com o objetivo de atender alguma requisição feita pelo
// usuário.
#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum ClientProtocol {
    #[serde(rename = "send_message")]
    SendMessage { from: String, to: String, text: String },

    #[serde(rename = "request_authenticate")]
    RequestAuthenticate { username: String, password: String },

    #[serde(rename = "create_user")]
    CreateUser { username: String, password: String },

    /* 
    Protocols a implementar:
    RequestFeed,
    */
}

// Protocolos enviados pelo server ao client com
// o objetivo de informá-lo sobre o status
// de requisições feitas pelo usuário.
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ServerProtocol {
    #[serde(rename = "message")]
    Message { from: String, to: String, text: String },

    #[serde(rename = "user_disconnected")]
    UserDisconnected { username: String },

    #[serde(rename = "error")]
    Error { error: ProtocolError },

    #[serde(rename = "authenticated")]
    Authenticated,

    #[serde(rename = "user_created")]
    UserCreated,

    // Protocolo especial que serve
    // apenas "comunicar" o proprio servidor
    // que algo pedido pelo cliente foi
    // satisfeito. Digo "comunicar" pois
    // na verdade Success nunca é lido,
    // ele serve apenas como valor de retorno.
    // Então Success é semanticamente equivalente a ().
    #[serde(rename = "success")]
    Success,

    /* 
    Protocols a implementar:
    Feed,
    */
}

pub enum InternalProtocol {
    OfflineMessage { username: String }
}

impl Protocol for ClientProtocol {}
impl Protocol for ServerProtocol {}
