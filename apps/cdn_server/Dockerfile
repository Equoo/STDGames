FROM rust:latest

WORKDIR /usr/src/app

COPY . .

RUN cargo build --release

CMD ["./target/release/server"]
