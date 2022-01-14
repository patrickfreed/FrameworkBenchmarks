mod models;
mod utils;

use std::ops::Deref;

use models::Fortune;

use actix_http::{
    body::BoxBody,
    header::{HeaderValue, CONTENT_TYPE, SERVER},
    KeepAlive, StatusCode,
};
use actix_web::{App, HttpResponse, HttpServer, middleware::Logger, web::{self, Bytes}};
use anyhow::{bail, Result};
use futures::TryStreamExt;
use mongodb::{options::ClientOptions, Client};
use yarte::ywrite_html;
use serde_json::json;
use log::info;

#[actix_web::get("/hello")]
async fn hello(data: web::Data<Client>) -> HttpResponse {
    HttpResponse::Ok().json(json!({"ok": 1}))
}

#[actix_web::get("/fortunes")]
async fn fortune(data: web::Data<Client>) -> HttpResponse {
    async fn fetch_fortunes(client: &Client) -> Result<Vec<Fortune>> {
        let mut fortunes: Vec<Fortune> = client
            .database("hello_world")
            .collection::<Fortune>("fortune")
            .find(None, None)
            .await?
            .try_collect()
            .await?;

        fortunes.push(Fortune {
            id: 0,
            message: "Additional fortune added at request time.".to_string(),
        });

        fortunes.sort_by(|a, b| a.message.cmp(&b.message));

        Ok(fortunes)
    }

    let res = fetch_fortunes(&data).await;

    match res {
        Ok(fortunes) => {
            let mut body = Vec::with_capacity(2048);
            ywrite_html!(body, "{{> fortune }}");

            let mut res = HttpResponse::with_body(StatusCode::OK, BoxBody::new(Bytes::from(body)));
            res.headers_mut()
                .insert(SERVER, HeaderValue::from_static("Actix"));
            res.headers_mut().insert(
                CONTENT_TYPE,
                HeaderValue::from_static("text/html; charset=utf-8"),
            );
            res
        }
        Err(e) => {
            HttpResponse::InternalServerError().body(e.to_string()).into()
        }
    }
}

#[actix_web::main]
async fn main() -> Result<()> {
    println!("Starting http server: 0.0.0.0:8080");
    // std::env::set_var("RUST_LOG", "debug");
    // std::env::set_var("RUST_BACKTRACE", "1");
    // env_logger::init();

    let uri = std::env::var("ACTIX_TECHEMPOWER_MONGODB_URL")
        .or_else(|_| bail!("missing ACTIX_TECHEMPOWER_MONGODB_URL env variable"))?;
    let mut options = ClientOptions::parse(uri).await?;
    options.min_pool_size = Some(56);
    options.max_pool_size = Some(56);
    let client = Client::with_options(options)?;

    HttpServer::new(move || {
        App::new()
            // .wrap(Logger::default())
            .app_data(web::Data::new(client.clone()))
            .service(fortune)
    })
    .keep_alive(KeepAlive::Os)
    .client_timeout(0)
    .backlog(1024)
    .bind("0.0.0.0:8080")?
    .run()
    .await?;

    Ok(())
}
