FROM rust:1 as builder

WORKDIR /usr/src/app

RUN rustup target add x86_64-unknown-linux-musl

COPY Cargo.toml Cargo.lock ./
COPY . .

RUN cargo build --release \
    --package snakerobots-server \
    --target x86_64-unknown-linux-musl

FROM scratch

COPY --from=builder /usr/src/app/target/x86_64-unknown-linux-musl/release/snakerobots-server /usr/local/bin/snakerobots-server

CMD [ "snakerobots-server" ]