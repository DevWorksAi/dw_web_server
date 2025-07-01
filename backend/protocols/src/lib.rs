use serde::{
    Deserialize,
    Serialize,
};

use error::{
	ProtocolError
};

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

    /* 
    Protocols a implementar:
    Feed,
    */
}
