FROM lukemathwalker/cargo-chef:latest-rust-1 AS chef
WORKDIR /usr/src/app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
RUN rustup target add x86_64-unknown-linux-musl
COPY --from=planner /usr/src/app/recipe.json recipe.json
RUN cargo chef cook --release \
    --package snakerobots-server \
    --target x86_64-unknown-linux-musl \
    --recipe-path recipe.json
COPY . .
RUN cargo build --release \
    --package snakerobots-server \
    --target x86_64-unknown-linux-musl

FROM scratch
COPY --from=builder /usr/src/app/target/x86_64-unknown-linux-musl/release/snakerobots-server /usr/local/bin/
CMD [ "/usr/local/bin/snakerobots-server" ]
