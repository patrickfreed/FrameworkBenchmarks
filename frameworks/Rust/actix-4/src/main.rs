use std::io;

use actix_http::{body::BoxBody, KeepAlive};
use actix_web::{
    http::{
        header::{HeaderValue, CONTENT_TYPE, SERVER},
        StatusCode,
    },
    web::{self, BufMut, Bytes, BytesMut},
    App, HttpResponse, HttpServer,
};
use anyhow::Result;
use simd_json_derive::Serialize;

pub const SIZE: usize = 27;

pub struct Writer<'a>(pub &'a mut BytesMut);

impl<'a> io::Write for Writer<'a> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.0.put_slice(buf);
        Ok(buf.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

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

#[actix_web::main]
async fn main() -> Result<()> {
    println!("Started http server: 127.0.0.1:8080");

    HttpServer::new(move || {
        App::new()
            .service(web::resource("/json").to(json))
            .service(web::resource("/plaintext").to(plaintext))
    })
    .keep_alive(KeepAlive::Os)
    .client_timeout(0)
    .backlog(1024)
    .bind("0.0.0.0:8080")?
    .run()
    .await?;

    Ok(())
}
