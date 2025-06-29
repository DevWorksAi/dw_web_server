use sqlx::{
    mysql::MySqlPool,
};

use serde::{
    Deserialize,
    Serialize,
};

use argon2::{
    {Argon2, PasswordHasher, PasswordHash, PasswordVerifier},
    password_hash::SaltString,
};

use rand::{
    rngs::OsRng,
};

use std::{
    fmt,
    path::PathBuf,
    env,
};

use dotenvy::from_path;

#[derive(Debug, Serialize, Deserialize)]
pub enum Error {
    Std,
    Hash,
    Sql,
    Envy,
    PasswordMismatch,
    UserNotFound,
    UserNotAdded,
    UserAlreadyExists,
}

impl From<argon2::password_hash::Error> for Error {
    fn from(_: argon2::password_hash::Error) -> Self {
        Self::Hash
    }
}

impl From<sqlx::Error> for Error {
    fn from(_: sqlx::Error) -> Self {
        Self::Sql
    }
}

impl From<dotenvy::Error> for Error {
    fn from(_: dotenvy::Error) -> Self {
        Self::Envy
    }
}

impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Std => write!(f, "Erro de std"),
            Error::Hash => write!(f, "Erro de argon2"),
            Error::Envy => write!(f, "Erro de dotenvy"),
            Error::Sql => write!(f, "Erro de slqx"),
            Error::PasswordMismatch => write!(f, "Senha inválida"),
            Error::UserNotFound => write!(f, "Usuário não encontrado"),
            Error::UserNotAdded => write!(f, "Usuário não foi cadastrado"),
            Error::UserAlreadyExists => write!(f, "Usuário já está cadastrado"),
        }
    }
}

#[derive(Debug)]
pub struct User {
    pub id: i32,
    pub username: String,
    pub password_hash: String,
}

pub async fn authenticate_user
(
    pool: &MySqlPool, 
    username: &str,
    password: &str,
) -> Result<(), Error>
{
    if let Some(row) = sqlx::query!(
        "SELECT id, username, password_hash FROM users WHERE username = ?",
        username,
    )
    .fetch_optional(pool)
    .await? {
        let hash_found = row.password_hash;
        let valid_user = check_password(&hash_found, password)?;

        match valid_user {
            true => Ok(()),
            false => Err(Error::PasswordMismatch),
        }        
    } else {
        Err(Error::UserNotFound)
    }
}

pub async fn add_user
(
    pool: &MySqlPool,
    username: &str,
    password: &str,
) -> Result<(), Error>
{
    let password_hash = hash_password(password)?;

    match sqlx::query!(
        "INSERT INTO users (username, password_hash) VALUES (?, ?)",
        username,
        password_hash,
    )
    .fetch_optional(pool)
    .await {
        Ok(_) => {},
        Err(_) => return Err(Error::UserAlreadyExists),
    };

    Ok(())
}

pub async fn remove_user
(
    pool: &MySqlPool,
    username: &str,
) -> Result<(), sqlx::Error>
{
    sqlx::query!(
        "DELETE FROM users WHERE username = ?",
        username,
    )
    .execute(pool)
    .await?;

    Ok(())
}

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

pub async fn connect_to_database() -> Result<MySqlPool, Error> {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.pop();
    path.push(".env");

    from_path(path.as_path()).unwrap();

    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL não definida");
    let pool = MySqlPool::connect(&db_url).await?;

    Ok(pool)    
}