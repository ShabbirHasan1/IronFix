# Build stage
FROM rust:1.92.0-alpine3.23 AS builder

RUN apk add --no-cache musl-dev

WORKDIR /app

COPY . .

RUN cargo build --release --example fix40_server -p ironfix-example

# Runtime stage
FROM alpine:3.23

RUN apk add --no-cache ca-certificates

WORKDIR /app

COPY --from=builder /app/target/release/examples/fix40_server /app/fix40_server

ENV FIX_HOST=0.0.0.0
ENV FIX_PORT=9870
ENV FIX_SENDER=SERVER
ENV FIX_TARGET=CLIENT

EXPOSE 9870

CMD ["/app/fix40_server"]
