# Build stage
FROM rust:1.92.0-alpine3.23 AS builder

RUN apk add --no-cache musl-dev

WORKDIR /app

COPY . .

RUN cargo build --release --example fast_server -p ironfix-example

# Runtime stage
FROM alpine:3.23

RUN apk add --no-cache ca-certificates

WORKDIR /app

COPY --from=builder /app/target/release/examples/fast_server /app/fast_server

ENV FAST_HOST=0.0.0.0
ENV FAST_PORT=9890

EXPOSE 9890

CMD ["/app/fast_server"]
