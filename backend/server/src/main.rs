use std::{
    net::SocketAddr,
};

use tokio::{
    io::Result,
};

use axum::{
    Router,
    routing::any,
};

use server::{
    handle::handle_connections::handler,
};

use users::{
    Users,
};

#[tokio::main]
async fn main() -> Result<()> {
    let users = Users::new();

    // cria a estrutura do server
    let app = Router::new()
        .route("/ws", any(handler))
        .with_state(users);

    // ouve via tcp no endereço dado
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    let listener = tokio::net::TcpListener::bind(addr)
        .await?;

    println!("Server rodando em ws::/{addr}");

    // cria efetivamente o servidor web
    axum::serve(listener,
        app.into_make_service_with_connect_info::<SocketAddr>())
        .await?;

    Ok(())
}
