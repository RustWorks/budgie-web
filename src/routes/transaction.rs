use actix_web::{error, web, HttpResponse, Result, Error, HttpRequest};
use actix_session::Session;

use serde::{Serialize, Deserialize};

use std::sync::Arc;

use sqlx::mysql::MySqlPool;

use chrono::{
    DateTime,
    offset::Utc,
};

use crate::{
    consts::TRANSACTION_PAGE_SIZE,
    get_user_id, respond_with_json,
};
use sqlx::executor::RefExecutor;
use sqlx::MySql;


#[derive(Serialize, Deserialize)]
pub struct JsonTransaction {
    id: u64,
    fund_source_id: u32,
    budget_id: Option<u32>,

    volume: i32,
    original_currency: String,

    notes: Option<String>,

    created_at: DateTime<Utc>,
}

#[derive(Deserialize)]
pub struct JsonCreateTransaction {
    volume: i32,
    notes: Option<String>,
}

#[derive(Deserialize)]
pub struct JsonGetTransactionParams {
    pub page: u32,
}

struct UserId {
    user_id: u32,
}

enum TransactionContext {
    Budget,
    FundSource,
}

impl TransactionContext {
    pub async fn check_user_id<'e, E: Send + RefExecutor<'e, Database = MySql>>(&self, mysql_pool: E, user_id: u32, source_id: u32) -> bool {
        match match self {
            Self::Budget => {
                sqlx::query_as!(UserId,
                    "SELECT fund_sources.user_id AS user_id FROM budgets INNER JOIN fund_sources ON budgets.fund_source_id = fund_sources.id WHERE budgets.id = ?",
                    source_id
                )
                    .fetch_one(mysql_pool)
                    .await
            },

            Self::FundSource => {
                sqlx::query_as!(UserId,
                    "SELECT user_id FROM fund_sources WHERE id = ?",
                    source_id
                )
                    .fetch_one(mysql_pool)
                    .await
            },
        } {
            Ok(row) => row.user_id == user_id,

            Err(_) => false,
        }
    }
}

impl TransactionContext {
    pub fn from_request(req: &HttpRequest) -> Result<Self, Error> {
        match req.match_info().get("type").unwrap() {
            "budget" => {
                Ok(TransactionContext::Budget)
            },

            "fund_source" => {
                Ok(TransactionContext::FundSource)
            },

            _ => {
                Err(error::ErrorNotFound("Page not found"))
            },
        }
    }
}

pub async fn create_transaction(req: HttpRequest, new_transaction: web::Json<JsonCreateTransaction>, pool: web::Data<Arc<MySqlPool>>, session: Session) -> Result<HttpResponse, Error> {
    let context = TransactionContext::from_request(&req)?;

    let context_id = req.match_info().get("id").unwrap().parse::<u32>().map_err(|_| error::ErrorBadRequest("ID must be integer"))?;

    let mut mysql_pool = pool.clone().acquire().await
        .map_err(|_| error::ErrorInternalServerError("SQLx obtaining error"))?;

    let user_id = get_user_id(session)?;

    if context.check_user_id(&mut mysql_pool, user_id, context_id).await {
        match match context {
            TransactionContext::Budget => {
                sqlx::query!(
                    "INSERT INTO transactions (fund_source_id, budget_id, volume, notes) VALUES ((SELECT fund_source_id FROM budgets WHERE id = ?), ?, ?, ?)",
                    context_id, context_id, new_transaction.volume, new_transaction.notes
                )
            },

            TransactionContext::FundSource => {
                sqlx::query!(
                    "INSERT INTO transactions (fund_source_id, volume, notes) VALUES (?, ?, ?)",
                    context_id, new_transaction.volume, new_transaction.notes
                )
            },
        }
            .execute(mysql_pool)
            .await {

            Ok(_) => {
                Ok(HttpResponse::Ok().content_type("application/json").body(r#"{"message": "Transaction created"}"#))
            },

            Err(_) => {
                Err(error::ErrorInternalServerError("SQLx query error"))
            },
        }
    }
    else {
        Err(error::ErrorBadRequest("No appropriate fund_source or budget with that ID"))
    }
}

pub async fn get_transactions(req: HttpRequest, parameters: web::Json<JsonGetTransactionParams>, pool: web::Data<Arc<MySqlPool>>, session: Session) -> Result<HttpResponse, Error> {
    let context = TransactionContext::from_request(&req)?;

    let context_id = req.match_info().get("id").unwrap().parse::<u32>().map_err(|_| error::ErrorBadRequest("ID must be integer"))?;

    let mut mysql_pool = pool.clone().acquire().await
        .map_err(|_| error::ErrorInternalServerError("SQLx obtaining error"))?;

    let user_id = get_user_id(session)?;

    if context.check_user_id(&mut mysql_pool, user_id, context_id).await {
        match match context {
            TransactionContext::Budget => {
                sqlx::query_as!(JsonTransaction,
                    "SELECT * FROM transactions WHERE budget_id = ? LIMIT ?, ?",
                    context_id, parameters.page, TRANSACTION_PAGE_SIZE
                )
                    .fetch_all(&mut mysql_pool)
                    .await
            },

            TransactionContext::FundSource => {
                sqlx::query_as!(JsonTransaction,
                    "SELECT * FROM transactions WHERE fund_source_id = ? LIMIT ?, ?",
                    context_id, parameters.page, TRANSACTION_PAGE_SIZE
                )
                    .fetch_all(&mut mysql_pool)
                    .await
            },
        } {

            Ok(rows) => {
                respond_with_json(&rows)
            },

            Err(_) => {
                Ok(HttpResponse::Ok().content_type("application/json").body(r#"[]"#))
            },
        }
    }
    else {
        Err(error::ErrorBadRequest("No appropriate fund_source or budget with that ID"))
    }
}
