# syntax=docker/dockerfile:1
# chronolexografie: een uitvoeringsproces naspelen op de cel van een fictieve
# uitvoerder. De bezoeker kiest een casus: vakantieverlof (Leerplichtwet 1969
# art. 13a) of de gehandicaptenparkeerkaart (BABW art. 49).
#
# Eigen component, niet gepubliceerd: alleen het poc-portaal bereikt hem, op
# http://{deployment}-chronolexografie:8000. Hij draait de cel (packages/cel) en
# de engine in een eigen proces, met de kroniek in het geheugen, en past
# daarom niet in het statische poc-image. Zelfde opzet als packages/poc-napp.

# Stage 1: de frontend.
FROM node:26-alpine@sha256:2d984a15c9b54fd0aeb608b8e0d0d83529eb34d2966db27a1fb4f1edc3d298a3 AS frontend-builder
WORKDIR /app
COPY package.json package-lock.json .npmrc ./
COPY packages/frontend-shared/package.json packages/frontend-shared/
COPY frontend-poc-chronolexografie/package.json frontend-poc-chronolexografie/
RUN npm ci --ignore-scripts
COPY packages/frontend-shared/ packages/frontend-shared/
COPY frontend-poc-chronolexografie/ frontend-poc-chronolexografie/
# Zelfde voorvoegsel als CHRONO_BASE_PATH hieronder: het portaal stuurt het pad
# ongewijzigd door.
RUN POC_BASE=/chronolexografie/ npm run build -w poc-chronolexografie

# Stage 2: chef-basis.
# Rust version: keep in sync with rust-toolchain.toml (source of truth).
FROM rust:1.96-alpine@sha256:a41f7740f8b45d45795624eec13a8b42263cc700f19f7e4e86e04d3dda08a479 AS chef
RUN apk add --no-cache musl-dev curl xz
ARG TARGETARCH
ARG CARGO_CHEF_VERSION=0.1.77
ARG CARGO_CHEF_SHA256_AMD64=a3733ab416c3ffddd37914cd13919ca05fee1a1cf654f3016dcfe7f399d89cd1
ARG CARGO_CHEF_SHA256_ARM64=3bff2e7ff979db837c7e820dfd8d51d2d97b9c94283506c6e596ae4887ef178b
RUN set -eu; \
    case "${TARGETARCH:-}" in \
      amd64) arch=x86_64; sha="${CARGO_CHEF_SHA256_AMD64}" ;; \
      arm64) arch=aarch64; sha="${CARGO_CHEF_SHA256_ARM64}" ;; \
      *) echo "geen gepinde cargo-chef-binary voor TARGETARCH='${TARGETARCH:-}'" >&2; exit 1 ;; \
    esac; \
    dir="cargo-chef-${arch}-unknown-linux-musl"; \
    curl -fsSLo "/tmp/${dir}.tar.xz" \
      "https://github.com/LukeMathWalker/cargo-chef/releases/download/v${CARGO_CHEF_VERSION}/${dir}.tar.xz"; \
    echo "${sha}  /tmp/${dir}.tar.xz" | sha256sum -c -; \
    tar -xJf "/tmp/${dir}.tar.xz" -C /tmp; \
    install -m0755 "/tmp/${dir}/cargo-chef" /usr/local/bin/cargo-chef; \
    rm -rf "/tmp/${dir}.tar.xz" "/tmp/${dir}"; \
    cargo chef --version
WORKDIR /build

# Stage 3: dependencies plannen.
FROM chef AS planner
COPY packages/Cargo.lock packages/Cargo.toml ./
RUN grep -qE '^members = \[$' Cargo.toml && \
    sed -i '/"admin"/d; /"arch-extract"/d; /"auth"/d; /"corpus"/d; /"editor-api"/d; /"github"/d; /"harvester"/d; /"pipeline"/d; /"poc-napp"/d; /"poc-portal"/d; /"tui"/d' Cargo.toml
COPY packages/cel/ cel/
COPY packages/engine/ engine/
COPY packages/law-model/ law-model/
COPY packages/poc-chronolexografie/ poc-chronolexografie/
COPY packages/shared/ shared/
RUN cargo chef prepare --recipe-path recipe.json

# Stage 4: de binary.
FROM chef AS builder
COPY packages/Cargo.lock ./
COPY --from=planner /build/Cargo.toml ./Cargo.toml
COPY --from=planner /build/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json
# `cargo chef cook` schrijft de workspace-Cargo.toml zonder
# `[workspace.lints]`, wat members met `[lints] workspace = true` breekt.
COPY --from=planner /build/Cargo.toml ./Cargo.toml
COPY packages/cel/ cel/
COPY packages/engine/ engine/
COPY packages/law-model/ law-model/
COPY packages/poc-chronolexografie/ poc-chronolexografie/
COPY packages/shared/ shared/
COPY schema/ /schema/
RUN find . -name '*.rs' -exec touch {} + && \
    cargo build --release --bin regelrecht-poc-chronolexografie

# Stage 5: runtime.
FROM alpine:3.24@sha256:28bd5fe8b56d1bd048e5babf5b10710ebe0bae67db86916198a6eec434943f8b
RUN apk add --no-cache ca-certificates && \
    addgroup -S app && adduser -S app -G app

COPY --from=builder /build/target/release/regelrecht-poc-chronolexografie /usr/local/bin/
COPY --from=frontend-builder /app/frontend-poc-chronolexografie/dist /app/static
# Alleen de regelwerken die de casussen draaien, en hun cellen met hun
# reducties en stromen van executogrammen. De app leidt per casus af welke
# regelwerken dat zijn (src/casus.rs); de test the_image_and_its_deploy_filter_hold_what_every_case_runs
# in src/kern.rs houdt deze lijst en script/deploy-filters.mjs daarmee gelijk.
COPY packages/poc-chronolexografie/casussen/ /app/casussen/
COPY corpus/regulation/nl/wet/leerplichtwet_1969/ /app/corpus/regulation/nl/wet/leerplichtwet_1969/
COPY corpus/regulation/nl/wet/algemene_wet_bestuursrecht/ /app/corpus/regulation/nl/wet/algemene_wet_bestuursrecht/
COPY corpus/regulation/nl/uitvoeringsbeleid/fictief_beleid_schooljaar_obs_de_klimop/ /app/corpus/regulation/nl/uitvoeringsbeleid/fictief_beleid_schooljaar_obs_de_klimop/
COPY corpus/regulation/nl/uitvoeringsbeleid/fictief_beleid_schooljaargrenzen_obs_de_klimop/ /app/corpus/regulation/nl/uitvoeringsbeleid/fictief_beleid_schooljaargrenzen_obs_de_klimop/
COPY corpus/regulation/nl/uitvoeringsbeleid/fictief_beleid_vakantierooster_obs_de_klimop/ /app/corpus/regulation/nl/uitvoeringsbeleid/fictief_beleid_vakantierooster_obs_de_klimop/
COPY corpus/regulation/nl/ministeriele_regeling/regeling_vaststelling_schoolvakanties_20252030/ /app/corpus/regulation/nl/ministeriele_regeling/regeling_vaststelling_schoolvakanties_20252030/
COPY corpus/cells/obs_de_klimop/ /app/corpus/cells/obs_de_klimop/
COPY corpus/regulation/nl/amvb/besluit_administratieve_bepalingen_inzake_het_wegverkeer_babw/ /app/corpus/regulation/nl/amvb/besluit_administratieve_bepalingen_inzake_het_wegverkeer_babw/
COPY corpus/regulation/nl/ministeriele_regeling/regeling_gehandicaptenparkeerkaart/ /app/corpus/regulation/nl/ministeriele_regeling/regeling_gehandicaptenparkeerkaart/
COPY corpus/regulation/nl/uitvoeringsbeleid/fictief_beleid_gehandicaptenparkeerkaart_gemeente_wilgendaal/ /app/corpus/regulation/nl/uitvoeringsbeleid/fictief_beleid_gehandicaptenparkeerkaart_gemeente_wilgendaal/
COPY corpus/cells/gemeente_wilgendaal/ /app/corpus/cells/gemeente_wilgendaal/

WORKDIR /app
ENV CHRONO_CORPUS_DIR=/app/corpus \
    CHRONO_CASUS_DIR=/app/casussen \
    CHRONO_STATIC_DIR=/app/static \
    CHRONO_PORT=8000 \
    CHRONO_BASE_PATH=/chronolexografie/
USER app
EXPOSE 8000
CMD ["regelrecht-poc-chronolexografie"]
