use serde::{
    Deserialize,
    Serialize,
};

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum ClientProtocol {
    #[serde(rename = "send_message")]
    SendMessage { text: String },

    #[serde(rename = "join_chat")]
    JoinChat { username: String },
    /* 
    Protocols a implementar:
    RequestFeed,
    ReqAuthenticate
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
    Error { message: String },

    /* 
    Protocols a implementar:
    Feed,
    Authenticate
    */
}
