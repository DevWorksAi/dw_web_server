/*
Estruturas e seus métodos lidar com usuários enquanto
User e enquanto valor guardado no database mysql.
*/

use std::{
    collections::HashMap,
    sync::Arc,
    path::PathBuf,
    env,
};

use tokio::{
    sync::{Mutex, mpsc::UnboundedSender},
};

use axum::{
    extract::ws::Message,
};

use sqlx::{
    mysql::MySqlPool,
};

use argon2::{
    {Argon2, PasswordHasher, PasswordHash, PasswordVerifier},
    password_hash::SaltString,
};

use rand::{
    rngs::OsRng,
};

use dotenvy::from_path;

use error::{
    AuthenticateErrorType,
};

type Tx = UnboundedSender<Message>;

// Tipo de usuário para tornar o código idiomático
#[derive(Eq, Hash, PartialEq, Clone)]
pub struct User {
    pub username: String,
}

impl User {
    pub fn new(username: &str) -> Self {
        Self {
            username: String::from(username),
        }
    }
}

// Tipo que contêm uma coleção de usuários o sender
// associado a suas conexões ao channel.
// É responsável, além de conter todos os usuário conectados
// ao client, ie, onlines, por adicionar um novo
// usuário na database mysql, remover um usuario do HashMap,
// hash de senhas para usuários, conectar na database etc.
#[derive(Clone)]
pub struct Users {
    pub on_users: Arc<Mutex<HashMap<User, Tx>>>,
}

impl Users {
    pub fn new() -> Self {
        Self {
            on_users: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    // Conecta na database registrada em .env
    async fn connect_to_database() -> Result<MySqlPool, AuthenticateErrorType> {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.pop();
        path.push(".env");

        from_path(path.as_path()).unwrap();

        let db_url = env::var("DATABASE_URL").expect("DATABASE_URL não definida");
        let pool = MySqlPool::connect(&db_url).await?;

        Ok(pool)    
    }

    // Retorna o sender do channel associado ao usuário.
    pub async fn get_user
    (
        &self,
        user: User,
    ) -> Option<Tx>
    {
        let on_users = self.on_users.lock().await;
        on_users.get(&user).cloned()
    }

    // Cria um novo usuário, se não existir, na database.
    pub async fn add_user
    (
        username: &str,
        password: &str,
    ) -> Result<(), AuthenticateErrorType>
    {
        let pool = Self::connect_to_database().await?;
        let password_hash = Self::hash_password(password)?;

        match sqlx::query!(
            "INSERT INTO users (username, password_hash) VALUES (?, ?)",
            username,
            password_hash,
        )
        .fetch_optional(&pool)
        .await {
            Ok(_) => {},
            Err(_) => return Err(AuthenticateErrorType::UserAlreadyExists),
        };

        Ok(())
    }

    // Remove um usuário de on_users.
    pub async fn remove_user
    (
        &mut self,
        username: &str,
    ) -> Option<Tx>
    {
        let mut on_users = self.on_users.lock().await;
        on_users.remove(&User::new(username))        
    }

    pub async fn user_exists
    (
        username: &str
    ) -> Result<bool, AuthenticateErrorType>
    {
        let pool = Self::connect_to_database().await?;

        let result = sqlx::query_scalar!(
            r#"
            SELECT EXISTS(
                SELECT 1 FROM users WHERE username = ?
            ) AS "exists!"
            "#,
            username,            
        )
        .fetch_one(&pool)
        .await?;

        Ok(result == 1)
    }

    // Faz o hash da senha passada pelo usuário
    // na hora da criação da conta.
    fn hash_password
    (
        password: &str,
    ) -> Result<String, argon2::password_hash::Error>
    {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();

        let password_hash = argon2.hash_password(password.as_bytes(), &salt)?;
        Ok(password_hash.to_string())
    }

    // Verifica se, ao tentar logar, a senha
    // passada pelo usuário é a mesma registrada
    // no database.
    fn check_password
    (
        hash_found: &str,
        password_given: &str,
    ) -> Result<bool, argon2::password_hash::Error>
    {
        let parsed_hash = PasswordHash::new(hash_found)?;
        Ok(Argon2::default().verify_password(password_given.as_bytes(),
            &parsed_hash)
            .is_ok()
        )
    }


    // Função responsável por autenticar/autorizar a entrada
    // do usuário na rede.
    pub async fn authenticate_user
    (
        &mut self,
        username: &str,
        password: &str,
        sender: Tx
    ) -> Result<(), AuthenticateErrorType>
    {
        let pool = Self::connect_to_database().await?;

        if let Some(row) = sqlx::query!(
            "SELECT id, username, password_hash FROM users WHERE username = ?",
            username,
        )
        .fetch_optional(&pool)
        .await? {
            let hash_found = row.password_hash;
            let valid_user = Self::check_password(&hash_found, password)?;

            match valid_user {
                true => {
                    let mut on_users = self.on_users.lock().await;
                    on_users.insert(User::new(username), sender);
                    Ok(())
                },
                false => Err(AuthenticateErrorType::PasswordMismatch),
            }        
        } else {
            Err(AuthenticateErrorType::UserNotFound)
        }
    }

    pub async fn store_message
    (
        sender: &str,
        receiver: &str,
        message: &str,
    ) -> Result<(), AuthenticateErrorType>
    {
        let pool = Self::connect_to_database().await?;

        match sqlx::query!(
            r#"
            INSERT INTO offline_messages (sender, receiver, message)
            VALUES (?, ?, ?)
            "#,
            sender,
            receiver,
            message,
        )
        .fetch_optional(&pool)
        .await {
            Ok(_) => Ok(()),
            Err(_) => Err(AuthenticateErrorType::OfflineMessageError)
        }
    }

    pub async fn get_stored_messages
    (
        receiver: &str,
    ) -> Result<Vec<(String, String)>, AuthenticateErrorType>
    {
        let pool = Self::connect_to_database().await?;

        let rows = sqlx::query!(
            r#"
            SELECT id, sender, message FROM offline_messages
            WHERE receiver = ?
            ORDER BY sent_at ASC
            "#,
            receiver
        )
        .fetch_all(&pool)
        .await?;

        let result = rows
            .into_iter()
            .map(|row| (row.sender, row.message))
            .collect::<Vec<_>>();

        Ok(result)
    }

    pub async fn delete_stored_messages
    (
        receiver: &str,
    ) -> Result<(), AuthenticateErrorType>
    {
        let pool = Self::connect_to_database().await?;

        sqlx::query!(
            r#"
            DELETE FROM offline_messages
            WHERE receiver = ?
            "#,
            receiver
        )
        .execute(&pool)
        .await?;

        Ok(())  
    }
}