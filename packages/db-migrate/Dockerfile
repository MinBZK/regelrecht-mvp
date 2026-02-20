# Stage 1: Build
FROM rust:1.85-alpine AS builder
RUN apk add --no-cache musl-dev

WORKDIR /build
COPY packages/db-migrate/ packages/db-migrate/

WORKDIR /build/packages/db-migrate
ENV SQLX_OFFLINE=true
RUN cargo build --release --bin regelrecht-db-migrate

# Stage 2: Runtime
FROM alpine:3.23
RUN apk add --no-cache ca-certificates && \
    addgroup -S app && adduser -S app -G app

COPY --from=builder /build/packages/db-migrate/target/release/regelrecht-db-migrate /usr/local/bin/

USER app
EXPOSE 8000
CMD ["regelrecht-db-migrate"]
