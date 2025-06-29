use utils::*;
use tokio::io::Result;

#[tokio::main]
async fn main() -> Result<()> {
    init_mysql_database(
        "root",
        "root",
        "nyoxon",
        "1234",
        "auth_database",
        "localhost",
        3306,
    ).await.expect("Erro ao tentar criar um database");

    Ok(())
}