# Build stage
FROM rust:1.92.0-alpine3.23 AS builder

RUN apk add --no-cache musl-dev

WORKDIR /app

COPY . .

RUN cargo build --release --example fix44_server -p ironfix-example

# Runtime stage
FROM alpine:3.23

RUN apk add --no-cache ca-certificates

WORKDIR /app

COPY --from=builder /app/target/release/examples/fix44_server /app/fix44_server

ENV FIX_HOST=0.0.0.0
ENV FIX_PORT=9876
ENV FIX_SENDER=SERVER
ENV FIX_TARGET=CLIENT

EXPOSE 9876

CMD ["/app/fix44_server"]
