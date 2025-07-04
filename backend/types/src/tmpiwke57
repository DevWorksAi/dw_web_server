/*
Aliases para tipos utéis em definições de
funções e métodos.
*/

use std::sync::Arc;
use tokio::sync::{
    mpsc::{UnboundedReceiver, UnboundedSender},
    Mutex,
};

use users::{
    Users,
    User,
};

use axum::extract::ws::{Message, WebSocket};

use futures_util::stream::{SplitSink, SplitStream};

pub type ArcWriter = Arc<Mutex<SplitSink<WebSocket, Message>>>;
pub type ArcReader = Arc<Mutex<SplitStream<WebSocket>>>;
pub type ArcUser = Arc<Mutex<User>>;
pub type ArcUsers = Arc<Mutex<Users>>;
pub type Tx = UnboundedSender<Message>;
pub type Rx = UnboundedReceiver<Message>;

