FROM rust:1.87-alpine

RUN apk add --no-cache openssl-dev ca-certificates pkgconfig musl-dev
COPY . .
RUN cargo install --locked --path .

ENTRYPOINT [ "warpgen" ]
