pub mod consts;
mod routes;

use actix_web::{
    error, middleware, web, App, HttpResponse, HttpServer, Result, Error,
    http::StatusCode, dev::ServiceResponse,
    middleware::errhandlers::{
        ErrorHandlerResponse, ErrorHandlers
    },
};
use actix_http::{body::Body, Response};
use actix_files::Files;
use actix_session::{
    CookieSession,
    Session,
};

use sqlx::mysql::MySqlPool;

use serde::Serialize;

use tera::Tera;

use std::{
    env,
    sync::Arc,
};

use routes::{
    user::{
        create_user, login_user, get_user_details,
    },
    fund_source::{
        create_fund_source, delete_fund_source, get_fund_source,
    },
    budget::{},
    transaction::{
        create_transaction, get_transactions,
    },
};


async fn index(tmpl: web::Data<Tera>) -> Result<HttpResponse, Error> {
    let s: String = tmpl.render("index.html", &tera::Context::new())
        .map_err(|_| error::ErrorInternalServerError("Template error"))?;

    Ok(HttpResponse::Ok().content_type("text/html").body(s))
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().unwrap();

    let mysql_pool = Arc::new(
        MySqlPool::new(&env::var("DATABASE_URL").expect("DATABASE_URL"))
            .await.unwrap());

    HttpServer::new(move || {
        let tera =
            Tera::new(concat!(env!("CARGO_MANIFEST_DIR"), "/templates/**/*")).unwrap();

        let pool = mysql_pool.clone();

        App::new()
            .data(tera)
            .data(pool)
            .wrap(middleware::Logger::default()) // enable logger
            .wrap(
                CookieSession::signed(env::var("SECRET").expect("SECRET not provided").as_bytes())
                    .secure(!cfg!(debug_assertions))
            )
            .service(web::resource("/").route(web::get().to(index)))
            .service(
                web::scope("/api")
                    .route("/user", web::post().to(create_user))
                    .route("/user", web::get().to(get_user_details))
                    .route("/user/login", web::get().to(login_user))

                    .route("/fund_source", web::post().to(create_fund_source))
                    .route("/fund_source/{fund_id}", web::delete().to(delete_fund_source))
                    .route("/fund_source/{fund_id}", web::get().to(get_fund_source))

                    .route("/{type}/{id}/transactions", web::post().to(create_transaction))
                    .route("/{type}/{id}/transactions", web::get().to(get_transactions))
            )
            .service(Files::new("/static", "./static/").show_files_listing())
            .service(web::scope("").wrap(error_handlers()))
    })
        .bind("127.0.0.1:5000")?
        .run()
        .await
}

fn error_handlers() -> ErrorHandlers<Body> {
    ErrorHandlers::new().handler(StatusCode::NOT_FOUND, not_found)
}

// Error handler for a 404 Page not found error.
fn not_found<B>(res: ServiceResponse<B>) -> Result<ErrorHandlerResponse<B>> {
    let response = get_error_response(&res, "Page not found");

    Ok(ErrorHandlerResponse::Response(
        res.into_response(response.into_body()),
    ))
}

// Generic error handler.
fn get_error_response<B>(res: &ServiceResponse<B>, error: &str) -> Response<Body> {
    let request = res.request();

    // Provide a fallback to a simple plain text response in case an error occurs during the
    // rendering of the error page.
    let fallback = |e: &str| {
        Response::build(res.status())
            .content_type("text/plain")
            .body(e.to_string())
    };

    let tera = request.app_data::<web::Data<Tera>>().map(|t| t.get_ref());

    match tera {
        Some(tera) => {
            let mut context = tera::Context::new();
            context.insert("error", error);
            context.insert("status_code", res.status().as_str());
            let body = tera.render("error.html", &context);

            match body {
                Ok(body) => Response::build(res.status())
                    .content_type("text/html")
                    .body(body),
                Err(_) => fallback(error),
            }
        }
        None => fallback(error),
    }
}

pub fn get_user_id(session: Session) -> Result<u32, Error> {
    if let Ok(user_id_opt) = session.get::<u32>("user_id") {
        if let Some(user_id) = user_id_opt {
            Ok(user_id)
        }
        else {
            Err(error::ErrorBadRequest("Not logged in"))
        }
    }
    else {
        Err(error::ErrorForbidden("Session corrupted"))
    }
}

pub fn respond_with_json<T: Serialize>(object: T) -> Result<HttpResponse, Error> {
    match serde_json::to_string(&object) {
        Ok(json) => {
            Ok(HttpResponse::Ok().content_type("application/json").body(json))
        },

        Err(e) => {
            Err(error::ErrorInternalServerError(format!("Could not produce JSON: {:?}", e)))
        },
    }
}
