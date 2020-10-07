use actix_web::{
    error, web, HttpResponse, Result, Error,
};
use actix_session::Session;

use serde::{Serialize, Deserialize};

use sha2::{Sha512, Digest};

use sqlx::mysql::MySqlPool;

use std::sync::Arc;

use chrono::{
    DateTime,
    offset::Utc,
};

use crate::{get_user_id, respond_with_json};


#[derive(Serialize, Deserialize)]
pub struct JsonCreateUser {
    username: String,
    email: String,
    password: String,
}

#[derive(Serialize, Deserialize)]
pub struct JsonUserDetails {
    id: u32,
    username: String,
    email: String,
    #[serde(skip_serializing)]
    password_hash: String,

    created_at: DateTime<Utc>,

    discord_id: Option<u64>,

    upgraded: bool,
    upgraded_at: Option<DateTime<Utc>>,
}

#[derive(Serialize, Deserialize)]
pub struct JsonLoginUser {
    email: String,
    password: String,
}

fn hash_password(password: impl AsRef<[u8]>, username: impl AsRef<[u8]>) -> String {
    let mut hasher = Sha512::default();

    hasher.update(password);
    hasher.update(b"+");
    hasher.update(username);

    hex::encode(hasher.finalize().as_slice())
}

fn check_password_hash(password_hash: String, password: impl AsRef<[u8]>, username: impl AsRef<[u8]>) -> bool {
    password_hash == hash_password(username, password)
}

pub async fn create_user(new_user: web::Json<JsonCreateUser>, pool: web::Data<Arc<MySqlPool>>) -> Result<HttpResponse, Error> {
    let mut mysql_pool = pool.clone().acquire().await
        .map_err(|_| error::ErrorInternalServerError("SQLx obtaining error"))?;

    // todo remove magic numbers
    if new_user.username.len() > 30 {
        Err(error::ErrorBadRequest("Username exceeds character limit (30)"))
    }
    else if new_user.email.len() > 254 {
        Err(error::ErrorBadRequest("Email exceeds character limit (254)"))
    }
    else {
        let existing_accounts = sqlx::query!(
            "SELECT username, email FROM users WHERE username = ? OR email = ?",
            new_user.username, new_user.email
        )
            .fetch_all(&mut mysql_pool)
            .await
            .map_err(|_| error::ErrorInternalServerError("SQLx query error"))?;

        if let Some(existing_user) = existing_accounts.get(0) {
            if existing_user.username == new_user.username {
                Err(error::ErrorBadRequest("Username already registered"))
            }
            else {
                Err(error::ErrorBadRequest("Email already registered"))
            }
        }
        else {
            let password_hash = hash_password(&new_user.password, &new_user.username);

            sqlx::query!(
                "INSERT INTO users (username, email, password_hash) VALUES (?, ?, ?)",
                new_user.username, new_user.email, password_hash
            )
                .execute(mysql_pool)
                .await
                .map_err(|_| error::ErrorInternalServerError("SQLx query error"))?;

            Ok(HttpResponse::Ok().content_type("application/json").body(r#"{"message": "User created"}"#))
        }
    }
}

pub async fn login_user(user_details: web::Json<JsonLoginUser>, pool: web::Data<Arc<MySqlPool>>, session: Session) -> Result<HttpResponse, Error> {
    let mut mysql_pool = pool.clone().acquire().await
        .map_err(|_| error::ErrorInternalServerError("SQLx obtaining error"))?;

    match sqlx::query!(
        "SELECT id, username, password_hash FROM users WHERE email = ?",
        user_details.email
    )
        .fetch_one(&mut mysql_pool)
        .await {

        Err(sqlx::Error::RowNotFound) => {
            println!("Email was not recognised");

            Err(error::ErrorBadRequest("Could not login with those credentials"))
        }

        Err(_) => {
            Err(error::ErrorInternalServerError("SQLx query error"))
        }

        Ok(user) => {
            if check_password_hash(user.password_hash.clone(), user.username, &user_details.password) {
                session.set("user_id", user.id).map_err(|_| error::ErrorInternalServerError("Could not set session variables up"))?;

                Ok(HttpResponse::Ok().content_type("application/json").body(r#"{"message": "Logged in"}"#))
            }
            else {
                println!("Password was incorrect");

                Err(error::ErrorBadRequest("Could not login with those credentials"))
            }
        }
    }
}

pub async fn get_user_details(pool: web::Data<Arc<MySqlPool>>, session: Session) -> Result<HttpResponse, Error> {
    let mut mysql_pool = pool.clone().acquire().await
        .map_err(|_| error::ErrorInternalServerError("SQLx obtaining error"))?;

    let user_id = get_user_id(session)?;

    match sqlx::query_as_unchecked!(JsonUserDetails,
        "SELECT * FROM users WHERE id = ?",
        user_id
    )
        .fetch_one(&mut mysql_pool)
        .await {

        Ok(user) => {
            respond_with_json(&user)
        },

        Err(sqlx::Error::RowNotFound) => {
            Err(error::ErrorBadRequest("Not logged in"))
        },

        Err(_) => {
            Err(error::ErrorInternalServerError("SQLx query failed"))
        },
    }
}
