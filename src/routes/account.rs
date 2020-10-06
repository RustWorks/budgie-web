use actix_web::{error, web, HttpResponse, Result, Error, HttpRequest};
use actix_session::Session;

use serde::{Serialize, Deserialize};

use std::sync::Arc;

use sqlx::mysql::MySqlPool;

use chrono::{
    DateTime,
    offset::Utc,
};


#[derive(Serialize, Deserialize)]
pub struct JsonCreateFundSource {
    pub name: String,
    pub default_currency: String,
}

#[derive(Serialize, Deserialize)]
pub struct JsonFundSource {
    id: u32,
    #[serde(skip_serialization)]
    user_id: u32,

    name: String,
    default_currency: String,

    created_at: DateTime<Utc>,
}

pub async fn create_fund_source(new_fund_source: web::Json<JsonCreateFundSource>, pool: web::Data<Arc<MySqlPool>>, session: Session) -> Result<HttpResponse, Error> {
    let mut mysql_pool = pool.clone().acquire().await
        .map_err(|_| error::ErrorInternalServerError("SQLx obtaining error"))?;

    if let Ok(user_id_opt) = session.get::<u32>("user_id") {
        if let Some(user_id) = user_id_opt {
            match sqlx::query!(
                "INSERT INTO fund_sources (name, default_currency, user_id) VALUES (?, ?, ?)",
                new_fund_source.name, new_fund_source.default_currency, user_id
            )
                .execute(&mysql_pool)
                .await {

                Ok(()) => {
                    Ok(HttpResponse::Ok().content_type("application/json").body(r#"{"message": "Fund source created"}"#))
                },

                Err(_) => {
                    Err(error::ErrorBadRequest("Not logged in"))
                },
            }
        }
        else {
            Err(error::ErrorBadRequest("Not logged in"))
        }
    }
    else {
        Err(error::ErrorForbidden("Session corrupted"))
    }
}

pub async fn delete_fund_source(req: HttpRequest, pool: web::Data<Arc<MySqlPool>>, session: Session) -> Result<HttpResponse, Error> {
    let fund_source_id = req.match_info().get("fund_id").unwrap();

    let mut mysql_pool = pool.clone().acquire().await
        .map_err(|_| error::ErrorInternalServerError("SQLx obtaining error"))?;

    if let Ok(user_id_opt) = session.get::<u32>("user_id") {
        if let Some(user_id) = user_id_opt {
            match sqlx::query!(
                "DELETE FROM fund_sources WHERE user_id = ? AND id = ?",
                new_fund_source.name, new_fund_source.default_currency, user_id
            )
                .execute(&mysql_pool)
                .await {

                Ok(()) => {
                    Ok(HttpResponse::Ok().content_type("application/json").body(r#"{"message": "Fund source deleted"}"#))
                },

                Err(_) => {
                    Err(error::ErrorBadRequest("Not logged in"))
                },
            }
        }
        else {
            Err(error::ErrorBadRequest("Not logged in"))
        }
    }
    else {
        Err(error::ErrorForbidden("Session corrupted"))
    }
}


pub async fn get_fund_source(req: HttpRequest, pool: web::Data<Arc<MySqlPool>>, session: Session) -> Result<HttpResponse, Error> {
    let fund_source_id = req.match_info().get("fund_id").unwrap();

    let mut mysql_pool = pool.clone().acquire().await
        .map_err(|_| error::ErrorInternalServerError("SQLx obtaining error"))?;

    if let Ok(user_id_opt) = session.get::<u32>("user_id") {
        if let Some(user_id) = user_id_opt {
            match sqlx::query_as!(JsonFundSource,
                "SELECT * FROM fund_sources WHERE user_id = ? AND id = ?",
                new_fund_source.name, new_fund_source.default_currency, user_id
            )
                .fetch_one(&mysql_pool)
                .await {

                Ok(fund_source) => {
                    match serde_json::to_string(fund_source) {
                        Ok(json) => {
                            Ok(HttpResponse::Ok().content_type("application/json").body(json))
                        },

                        Err(e) => {
                            Err(error::ErrorInternalServerError(format!("Could not produce JSON: {:?}", e)))
                        },
                    }
                }

                Err(sqlx::Error::RowNotFound) => {
                    Err(error::ErrorNotFound("No funding source with that ID"))
                },

                Err(_) => {
                    Err(error::ErrorInternalServerError("SQLx query failed"))
                },
            }
        }
        else {
            Err(error::ErrorBadRequest("Not logged in"))
        }
    }
    else {
        Err(error::ErrorForbidden("Session corrupted"))
    }
}
