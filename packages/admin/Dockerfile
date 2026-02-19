# Stage 1: Build frontend
FROM node:22-alpine AS frontend-builder
WORKDIR /app
COPY packages/admin/frontend-src/package.json packages/admin/frontend-src/package-lock.json* packages/admin/frontend-src/.npmrc ./
RUN --mount=type=secret,id=GITHUB_TOKEN \
    GITHUB_TOKEN=$(cat /run/secrets/GITHUB_TOKEN) npm ci
COPY packages/admin/frontend-src/ .
# vite outDir is '../static' which resolves to /static from /app
RUN npm run build

# Stage 2: Build Rust binary
FROM rust:1.84-alpine AS rust-builder
RUN apk add --no-cache musl-dev
WORKDIR /build
COPY packages/admin/ packages/admin/
WORKDIR /build/packages/admin
RUN cargo build --release --bin regelrecht-admin

# Stage 3: Runtime
FROM alpine:3.21
RUN apk add --no-cache ca-certificates && \
    addgroup -S app && adduser -S app -G app

COPY --from=rust-builder /build/packages/admin/target/release/regelrecht-admin /usr/local/bin/
COPY --from=frontend-builder /static /app/static

WORKDIR /app
USER app
EXPOSE 8000
CMD ["regelrecht-admin"]
