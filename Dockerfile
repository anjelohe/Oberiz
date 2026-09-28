# syntax=docker/dockerfile:1

FROM node:22-bookworm-slim AS frontend-build
WORKDIR /build/frontend
COPY frontend/package.json frontend/package-lock.json ./
RUN npm ci
COPY frontend/ ./
RUN npm run build

FROM rust:1.90-bookworm AS backend-build
WORKDIR /build/backend
COPY backend/Cargo.toml backend/Cargo.lock ./
COPY backend/src ./src
COPY backend/migrations ./migrations
RUN cargo build --release --locked

FROM debian:bookworm-slim
RUN apt-get update \
    && apt-get install --no-install-recommends -y ca-certificates \
    && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=backend-build /build/backend/target/release/oberiz /usr/local/bin/oberiz
COPY --from=frontend-build /build/frontend/dist ./frontend
COPY config ./config
RUN mkdir -p /data /config/indexers/custom /config/indexers/upstream \
    && useradd --system --uid 10001 --home /app --shell /usr/sbin/nologin oberiz \
    && chown -R oberiz:oberiz /app /data /config
USER oberiz
ENV OBERIZ_DATA_DIR=/data \
    OBERIZ_CONFIG_DIR=/config \
    OBERIZ_STATIC_DIR=/app/frontend
EXPOSE 2032
VOLUME ["/data", "/config"]
ENTRYPOINT ["oberiz"]
