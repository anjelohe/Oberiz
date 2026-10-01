# syntax=docker/dockerfile:1

# Pinned by digest, not just tag, for reproducible release builds. Dependabot
# (.github/dependabot.yml) opens a PR to bump these when a new patched image
# is published, so pinning doesn't quietly turn into running a stale image.
FROM node:22-bookworm-slim@sha256:43ac6c60b8f89723f746e8a92ce91abd5017e627ce1ddfe4238355d3a30b772c AS frontend-build
WORKDIR /build/frontend
COPY frontend/package.json frontend/package-lock.json ./
RUN npm ci
COPY frontend/ ./
RUN npm run build

FROM rust:1.98-bookworm@sha256:93ce27a88655056a51dbdd8f5f2d7ddc071c7b0070fb288a37b5a285fc83971e AS backend-build
WORKDIR /build/backend
COPY backend/Cargo.toml backend/Cargo.lock ./
COPY backend/src ./src
COPY backend/migrations ./migrations
RUN cargo build --release --locked

FROM debian:bookworm-slim@sha256:3783cc01769c7b2b1b83a5c5ad96c815348e28ed7da68e2e3687004faa906251
# Package versions aren't pinned here on purpose: the base image itself is
# pinned by digest above, and letting ca-certificates/curl take whatever
# security patch that fixed image's own apt repo currently has is preferable
# to freezing a specific .deb build that Debian's mirrors eventually delete,
# which would break every future build from this same pinned base.
# hadolint ignore=DL3008
RUN apt-get update \
    && apt-get install --no-install-recommends -y ca-certificates curl \
    && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=backend-build /build/backend/target/release/oberiz /usr/local/bin/oberiz
COPY --from=frontend-build /build/frontend/dist ./frontend
COPY config ./config
RUN mkdir -p /data /config/indexers/custom /config/indexers/upstream \
    && groupadd --system --gid 10001 oberiz \
    && useradd --system --uid 10001 --gid 10001 --home /app --shell /usr/sbin/nologin oberiz \
    && chown -R 10001:10001 /app /data /config
USER 10001:10001
ENV OBERIZ_DATA_DIR=/data \
    OBERIZ_CONFIG_DIR=/config \
    OBERIZ_STATIC_DIR=/app/frontend
EXPOSE 2032
VOLUME ["/data", "/config"]
HEALTHCHECK --interval=30s --timeout=3s --start-period=10s --retries=3 \
    CMD ["curl", "-fsS", "http://127.0.0.1:2032/api/health/live"]
ENTRYPOINT ["oberiz"]
