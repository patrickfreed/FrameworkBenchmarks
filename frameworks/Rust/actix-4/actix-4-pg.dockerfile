FROM rust:1.57.0

RUN apt-get update -yqq && apt-get install -yqq cmake g++

ADD ./ /actix
WORKDIR /actix

RUN cargo clean
RUN RUSTFLAGS="-C target-cpu=native" cargo build --release --bin actix-4-pg

EXPOSE 8080

CMD ./target/release/actix-4-pg