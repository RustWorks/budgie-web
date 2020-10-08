use actix_web::{error, web, HttpResponse, Result, Error, HttpRequest};
use actix_session::Session;

use serde::{Serialize, Deserialize};

use std::sync::Arc;

use sqlx::mysql::MySqlPool;

use chrono::{
    DateTime,
    offset::Utc,
};

use sqlx::types::BigDecimal;

use crate::{get_user_id, respond_with_json};


#[derive(Serialize, Deserialize)]
pub struct JsonCreateFundSource {
    pub name: String,
    pub default_currency: String,
}

#[derive(Serialize, Deserialize)]
pub struct JsonFundSource {
    id: u32,
    #[serde(skip_serializing)]
    user_id: u32,

    name: String,
    default_currency: String,

    created_at: DateTime<Utc>,

    pub balance: Option<BigDecimal>,
}

pub async fn create_fund_source(new_fund_source: web::Json<JsonCreateFundSource>, pool: web::Data<Arc<MySqlPool>>, session: Session) -> Result<HttpResponse, Error> {
    let mysql_pool = pool.clone().acquire().await
        .map_err(|_| error::ErrorInternalServerError("SQLx obtaining error"))?;

    let user_id = get_user_id(session)?;

    match sqlx::query!(
        "INSERT INTO fund_sources (name, default_currency, user_id) VALUES (?, ?, ?)",
        new_fund_source.name, new_fund_source.default_currency, user_id
    )
        .execute(mysql_pool)
        .await {

        Ok(_) => {
            Ok(HttpResponse::Ok().content_type("application/json").body(r#"{"message": "Fund source created"}"#))
        },

        Err(_) => {
            Err(error::ErrorBadRequest("Not logged in"))
        },
    }
}

pub async fn delete_fund_source(req: HttpRequest, pool: web::Data<Arc<MySqlPool>>, session: Session) -> Result<HttpResponse, Error> {
    let fund_source_id = req.match_info().get("fund_id").unwrap();

    let mysql_pool = pool.clone().acquire().await
        .map_err(|_| error::ErrorInternalServerError("SQLx obtaining error"))?;

    let user_id = get_user_id(session)?;

    match sqlx::query!(
        "DELETE FROM fund_sources WHERE user_id = ? AND id = ?",
        user_id, fund_source_id
    )
        .execute(mysql_pool)
        .await {

        Ok(_) => {
            Ok(HttpResponse::Ok().content_type("application/json").body(r#"{"message": "Fund source deleted"}"#))
        },

        Err(_) => {
            Err(error::ErrorBadRequest("Not logged in"))
        },
    }
}


pub async fn get_fund_source(req: HttpRequest, pool: web::Data<Arc<MySqlPool>>, session: Session) -> Result<HttpResponse, Error> {
    let fund_source_id = req.match_info().get("fund_id").unwrap();

    let mut mysql_pool = pool.clone().acquire().await
        .map_err(|_| error::ErrorInternalServerError("SQLx obtaining error"))?;

    let user_id = get_user_id(session)?;

    match sqlx::query_as!(JsonFundSource,
        "SELECT *, (SELECT SUM(volume) AS balance FROM transactions WHERE fund_source_id = ?) AS balance FROM fund_sources WHERE user_id = ? AND id = ?",
        fund_source_id, fund_source_id, user_id
    )
        .fetch_one(&mut mysql_pool)
        .await {

        Ok(fund_source) => {
            respond_with_json(&fund_source)
        }

        Err(sqlx::Error::RowNotFound) => {
            Err(error::ErrorNotFound("No funding source with that ID"))
        },

        Err(e) => {
            println!("{:?}", e);

            Err(error::ErrorInternalServerError("SQLx query failed"))
        },
    }
}
