# Build stage
FROM rust:1.92.0-alpine3.23 AS builder

RUN apk add --no-cache musl-dev

WORKDIR /app

COPY . .

RUN cargo build --release --example fix50sp2_server -p ironfix-example

# Runtime stage
FROM alpine:3.23

RUN apk add --no-cache ca-certificates

WORKDIR /app

COPY --from=builder /app/target/release/examples/fix50sp2_server /app/fix50sp2_server

ENV FIX_HOST=0.0.0.0
ENV FIX_PORT=9882
ENV FIX_SENDER=SERVER
ENV FIX_TARGET=CLIENT

EXPOSE 9882

CMD ["/app/fix50sp2_server"]
