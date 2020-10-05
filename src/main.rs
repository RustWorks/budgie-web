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
use actix_session::CookieSession;

use sqlx::mysql::MySqlPool;

use tera::Tera;

use std::{
    env,
    sync::Arc,
};

use routes::account::{
    create_account, login_account, get_account_details,
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
                    .route("/account", web::post().to(create_account))
                    .route("/account", web::get().to(get_account_details))
                    .route("/account/login", web::get().to(login_account))
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
