FROM rust:1.57.0

RUN apt-get update -yqq && apt-get install -yqq cmake g++

ADD ./ /actix-4
WORKDIR /actix-4

RUN cargo clean
RUN RUSTFLAGS="-C target-cpu=native" cargo build --release

EXPOSE 8080

CMD ./target/release/actix-4
