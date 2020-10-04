use actix_web::{
    error, web, HttpResponse, Result, Error,
};

use serde::{Serialize, Deserialize};

use sha2::{Sha512, Digest};

use std::sync::Arc;

use sqlx::mysql::MySqlPool;


#[derive(Serialize, Deserialize)]
pub struct JsonUserAccount {
    username: String,
    email: String,
    password: String,
}

#[derive(Serialize, Deserialize)]
pub struct JsonUserCredentials {
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

pub async fn create_account(new_user: web::Json<JsonUserAccount>, pool: web::Data<Arc<MySqlPool>>) -> Result<HttpResponse, Error> {
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

pub async fn login_account(user_details: web::Json<JsonUserCredentials>, pool: web::Data<Arc<MySqlPool>>) -> Result<HttpResponse, Error> {
    let mut mysql_pool = pool.clone().acquire().await
        .map_err(|_| error::ErrorInternalServerError("SQLx obtaining error"))?;

    match sqlx::query!(
        "SELECT username, password_hash FROM users WHERE email = ?",
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
            if check_password_hash(user.password_hash, user.username, &user_details.password) {
                Ok(HttpResponse::Ok().content_type("application/json").body(r#"{"message": "Logged in", "token": ""}"#))
            }
            else {
                println!("Password was incorrect");

                Err(error::ErrorBadRequest("Could not login with those credentials"))
            }
        }
    }
}
