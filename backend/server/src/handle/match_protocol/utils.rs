use error::{ProtocolError};

use protocols::{
    ServerProtocol,
    Protocol,
};

use types::Tx;

// Lida com cada tipo de ServerProtocol criado
// dentro de handle_protocol();
pub async fn handle_instance
(
    tx: Tx,
    instance: ServerProtocol,
)
{   
    // Se protocol conseguir ser enviado
    // então ele será enviado pelo try_send,
    // por isso não é considerado o caso
    // Ok() em handle_result, afinal sabemos
    // com certeza que será um Success.
    let result = instance.serialize_and(async |json| {
        try_send(
            tx.clone(), 
            &json).await;
        ServerProtocol::Success
    }).await;

    handle_result(tx, result).await;    
}

// Lida com o result retornado por serialize_and usado
// em handle_instance();
pub async fn handle_result
(
    tx: Tx,
    r: Result<ServerProtocol, ProtocolError>,
)
{
    if let Err(e) = r {
        let err = ServerProtocol::Error {
            error: e,
        };

        let result = err.serialize_and(async |json| {
            try_send(tx, &json).await;
        }).await;

        // Isto é importante! Não estou enviando para
        // o cliente erros do tipo Serde, mas apenas
        // imprimindo na saída de erro padrão do servidor.
        // Portanto, caso no futuro algo estranho aconteça enquanto
        // o servidor está online, é inteligente verificar
        // a stderr do server.
        if result.is_err() {
            eprintln!("Erro de serialização");
        }
    }
}

// Tenta enviar to_send pela socket
pub async fn try_send
(
    tx: Tx,
    to_send: &str,
) 
{
    if tx.send(to_send.into()).is_err() {
        eprintln!(
        "Erro ao tentar enviar pelo channel; Motivo: rx foi dropado");
    }
}