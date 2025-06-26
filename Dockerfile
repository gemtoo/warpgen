FROM alpine:3.22 AS base

FROM rust:1.87-alpine3.22 AS chef
RUN apk add --no-cache openssl-dev ca-certificates pkgconfig musl-dev
RUN cargo install cargo-chef --locked

FROM chef AS planner
WORKDIR /app
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
WORKDIR /app
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json
COPY . .
RUN cargo build --release --bin warpgen && \
    rm -rf target/release/deps target/release/build

FROM base AS warpgen
COPY --from=builder /app/target/release/warpgen /usr/local/bin/
ENTRYPOINT [ "warpgen" ]
