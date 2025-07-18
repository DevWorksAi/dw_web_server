/*
Funções com propósito geral. Essas funções devem ser
chamadas em /utils/src/main.rs
*/

use sqlx::{
    mysql::MySqlPoolOptions,
    Executor,
};

use std::{
    fs,
    path::PathBuf,
};

// Cria automaticamente, se não existir, a database de nome
// db_name no ip host e porta port. Além de dar permissões 
// de acceso ao database pro db_user e criar as tabelas 
// importantes dentrodo database.
// (atualmente só cria a de user/password)
pub async fn init_mysql_database(
    root_user: &str,
    root_pass: &str,
    db_user: &str,
    db_pass: &str,
    db_name: &str,
    host: &str,
    port: u16,
) -> Result<(), Box<dyn std::error::Error>> {
    let admin_url = format!("mysql://{}:{}@{}:{}/", root_user, root_pass, host, port);
    let admin_pool = MySqlPoolOptions::new()
        .connect(&admin_url)
        .await?;

    let create_user_sql = format!(
        "CREATE USER IF NOT EXISTS '{}'@'%' IDENTIFIED BY '{}'",
        db_user, db_pass
    );
    let _ = admin_pool.execute(create_user_sql.as_str()).await;

    let drop_user_localhost = format!("DROP USER IF EXISTS '{}'@'localhost'", db_user);
    admin_pool.execute(drop_user_localhost.as_str()).await?;

    let create_db_sql = format!("CREATE DATABASE IF NOT EXISTS `{}`", db_name);
    admin_pool.execute(create_db_sql.as_str()).await?;

    let grant_sql = format!(
        "GRANT ALL PRIVILEGES ON `{}`.* TO '{}'@'%'",
        db_name, db_user
    );
    admin_pool.execute(grant_sql.as_str()).await?;
    admin_pool.execute("FLUSH PRIVILEGES").await?;

    let user_url = format!("mysql://{}:{}@{}:{}/{}", db_user, db_pass, host, port, db_name);
    let user_pool = MySqlPoolOptions::new().connect(&user_url).await?;

    let create_users_table = r#"
        CREATE TABLE IF NOT EXISTS users (
            id INT AUTO_INCREMENT PRIMARY KEY,
            username VARCHAR(255) NOT NULL UNIQUE,
            password_hash TEXT NOT NULL
        );
    "#;
    user_pool.execute(create_users_table).await?;

    let create_offline_table = r#"
        CREATE TABLE IF NOT EXISTS offline_messages (
            id INT AUTO_INCREMENT PRIMARY KEY,
            sender VARCHAR(255) NOT NULL,
            receiver VARCHAR(255) NOT NULL,
            message TEXT NOT NULL,
            sent_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (sender) REFERENCES users(username) ON DELETE CASCADE,
            FOREIGN KEY (receiver) REFERENCES users(username) ON DELETE CASCADE
        );
    "#;
    user_pool.execute(create_offline_table).await?;

    let db_url = format!("mysql://{}:{}@{}:{}/{}", db_user, db_pass, host, port, db_name);

    let mut env_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    env_path.pop();
    env_path.push(".env");

    let contents = format!("DATABASE_URL={}\n", db_url);
    fs::write(&env_path, contents)?;

    println!("[OK] Banco `{}` e usuário `{}` configurados com sucesso!", db_name, db_user);
    println!("[OK] .env criado em {:?}", env_path);

    Ok(())
}
