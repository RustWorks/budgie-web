use actix_web::{
    error, web, HttpResponse, Result, Error,
};
use actix_session::Session;

use serde::{Serialize, Deserialize};

use std::sync::Arc;

use sqlx::mysql::MySqlPool;


#[derive(Serialize, Deserialize)]
pub struct JsonCreateFundSource {
    name: String,
    default_currency: String,
}

pub async fn create_fund_source(new_fund_source: web::Json<JsonCreateFundSource>, pool: web::Data<Arc<MySqlPool>>) -> Result<HttpResponse, Error> {

}
