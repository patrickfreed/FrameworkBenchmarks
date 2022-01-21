mod models;
mod utils;

use std::borrow::Borrow;

use actix_http::{body::BoxBody, KeepAlive};
use actix_web::{
    dev::{HttpServiceFactory, Server},
    http::{
        header::{HeaderValue, CONTENT_TYPE, SERVER},
        StatusCode,
    },
    web::{self, Bytes, BytesMut},
    App, HttpResponse, HttpServer,
};
use anyhow::{bail, Result};
use deadpool_postgres::{Config, Pool, PoolConfig, Runtime};
use futures::{FutureExt, StreamExt};
use simd_json_derive::Serialize;
use tokio_postgres::NoTls;
use utils::{Writer, SIZE};
use yarte::ywrite_html;

use crate::models::Fortune;

#[derive(Serialize)]
pub struct Message {
    pub message: &'static str,
}

async fn json() -> HttpResponse {
    let message = Message {
        message: "Hello, World!",
    };
    let mut body = BytesMut::with_capacity(SIZE);
    message.json_write(&mut Writer(&mut body)).unwrap();

    let mut res = HttpResponse::with_body(StatusCode::OK, BoxBody::new(body.freeze()));
    res.headers_mut()
        .insert(SERVER, HeaderValue::from_static("A"));
    res.headers_mut()
        .insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
    res
}

async fn plaintext() -> HttpResponse {
    let mut res = HttpResponse::with_body(
        StatusCode::OK,
        BoxBody::new(Bytes::from_static(b"Hello, World!")),
    );
    res.headers_mut()
        .insert(SERVER, HeaderValue::from_static("A"));
    res.headers_mut()
        .insert(CONTENT_TYPE, HeaderValue::from_static("text/plain"));
    res
}

#[actix_web::get("/fortunes")]
async fn fortune(data: web::Data<Pool>) -> HttpResponse {
    async fn fetch_fortunes(pool: &Pool) -> Result<Vec<Fortune>> {
        let conn = pool.get().await?;
        let stmt = conn.prepare("SELECT * FROM Fortune").await?;
        let params: &[&'static str] = &[];
        let s = conn.query_raw(&stmt, params).await?;
        let mut stream = Box::pin(s);
        let mut fortunes = Vec::new();

        while let Some(row) = stream.next().await {
            let row = row
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, format!("{:?}", e)))?;
            fortunes.push(Fortune {
                id: row.get(0),
                message: row.get(1),
            });
        }

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
        Err(e) => HttpResponse::InternalServerError()
            .body(e.to_string())
            .into(),
    }
}

#[actix_web::main]
async fn main() -> Result<()> {
    println!("Starting http server: 0.0.0.0:8080");
    // std::env::set_var("RUST_LOG", "debug");
    // std::env::set_var("RUST_BACKTRACE", "1");
    // env_logger::init();

    // let uri = std::env::var("ACTIX_TECHEMPOWER_MONGODB_URL")
    //     .or_else(|_| bail!("missing ACTIX_TECHEMPOWER_MONGODB_URL env variable"))?;

    // postgres://benchmarkdbuser:benchmarkdbpass@tfb-database/hello_world
    let mut cfg = Config::new();
    cfg.host = Some("tfb-database".to_string());
    cfg.dbname = Some("hello_world".to_string());
    cfg.user = Some("benchmarkdbuser".to_string());
    cfg.password = Some("benchmarkdbpass".to_string());
    let pc = PoolConfig::new(56);
    cfg.pool = pc.into();
    let pool = cfg.create_pool(Some(Runtime::Tokio1), NoTls).unwrap();

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(pool.clone()))
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
