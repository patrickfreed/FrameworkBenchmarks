mod models;
mod utils;

use std::ops::Deref;

use models::Fortune;

use actix_http::{
    body::BoxBody,
    header::{HeaderValue, CONTENT_TYPE, SERVER},
    KeepAlive, StatusCode,
};
use actix_web::{
    middleware::Logger,
    web::{self, Bytes},
    App, HttpResponse, HttpServer,
};
use anyhow::{bail, Result};
use futures::TryStreamExt;
use log::info;
use mongodb::bson::RawDocumentBuf;
use mongodb::{options::ClientOptions, Client};
use serde_json::json;
use yarte::ywrite_html;

#[actix_web::get("/hello")]
async fn hello(data: web::Data<Client>) -> HttpResponse {
    HttpResponse::Ok().json(json!({"ok": 1}))
}

#[actix_web::get("/fortunes")]
async fn fortune(data: web::Data<Client>) -> HttpResponse {
    async fn fetch_fortunes(client: &Client) -> Result<Vec<Fortune>> {
        let mut fortunes_cursor = client
            .database("hello_world")
            .collection::<RawDocumentBuf>("fortune")
            .find(None, None)
            .await?;
        let mut fortunes = Vec::new();

        // todo!()
        // while let Some(doc) = fortunes_cursor.try_next().await? {
        //     // let f = Fortune {
        //     //     id: doc.get_f64("id")? as i32,
        //     //     message: doc.get_str("message")?.to_string(),
        //     // };
        //     let mut iter = doc.into_iter();
        //     while let Some(Ok((k, v))) = iter.next() {
        //         match (k, v) {
        //             ("id", RawBsonRef::Double(d)) =>  
        //         }
        //     }
        //     fortunes.push(f);
        // }

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

fn main() {
    actix_web::rt::System::with_tokio_rt(|| tokio::runtime::Runtime::new().unwrap())
        .block_on(async_main())
        .unwrap();
}

async fn async_main() -> Result<()> {
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
