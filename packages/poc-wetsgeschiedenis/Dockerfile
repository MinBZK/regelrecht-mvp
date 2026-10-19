# syntax=docker/dockerfile:1
# De wetsgeschiedenis: de pagina, en een dienst die een wet ophaalt vanaf haar
# BWB-id en haar Kamerdossiers afspeelt. Draait als eigen ZAD-component
# (`wetsgeschiedenis`), niet gepubliceerd; het poc-portaal stuurt
# /wetsgeschiedenis/... ongewijzigd door, dus de dienst serveert zichzelf onder
# dat voorvoegsel (WG_BASE_PATH).
#
# Geen Rust en geen WASM: de dienst is Python (standaardbibliotheek, pyyaml) en
# haalt de stukken met curl. De basisimages zijn dezelfde, met dezelfde digest,
# als in packages/poc-portal/Dockerfile.

# Stage 1: de frontend, met het voorvoegsel van het portaal.
FROM node:26-alpine@sha256:2d984a15c9b54fd0aeb608b8e0d0d83529eb34d2966db27a1fb4f1edc3d298a3 AS frontend-builder
WORKDIR /app
COPY package.json package-lock.json .npmrc ./
COPY packages/frontend-shared/package.json packages/frontend-shared/
COPY frontend-poc-wetsgeschiedenis/package.json frontend-poc-wetsgeschiedenis/
RUN npm ci --ignore-scripts
COPY corpus-poc/wetsgeschiedenis/ corpus-poc/wetsgeschiedenis/
COPY frontend-poc-wetsgeschiedenis/ frontend-poc-wetsgeschiedenis/
RUN POC_BASE=/wetsgeschiedenis/ npm run build -w poc-wetsgeschiedenis

# Stage 2: de dienst.
FROM alpine:3.24@sha256:28bd5fe8b56d1bd048e5babf5b10710ebe0bae67db86916198a6eec434943f8b
RUN apk add --no-cache python3 py3-yaml curl ca-certificates && \
    addgroup -S app && adduser -S app -G app && \
    mkdir -p /data && chown app:app /data
WORKDIR /app
COPY packages/poc-wetsgeschiedenis/dienst.py ./
COPY packages/poc-wetsgeschiedenis/afspeler/ afspeler/
COPY corpus-poc/wetsgeschiedenis/gegevens/ vast/
COPY --from=frontend-builder /app/frontend-poc-wetsgeschiedenis/dist static/

# /data houdt wat bezoekers ophalen. Zonder volume is het na een herstart weg;
# de dossiers in vast/ blijven altijd.
ENV WG_PORT=8000 \
    WG_BASE_PATH=/wetsgeschiedenis/ \
    WG_STATIC_DIR=/app/static \
    WG_VAST_DIR=/app/vast \
    WG_DATA=/data \
    PYTHONDONTWRITEBYTECODE=1 \
    PYTHONUNBUFFERED=1
USER app
EXPOSE 8000
CMD ["python3", "dienst.py"]
