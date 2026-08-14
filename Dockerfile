# syntax=docker/dockerfile:1

# ── 前端构建 ──
FROM node:22-alpine AS frontend
WORKDIR /app/frontend
COPY frontend/package.json frontend/package-lock.json ./
RUN npm ci
COPY frontend/ ./
RUN npm run build

# ── 后端构建（完整 rust 镜像自带 gcc，供 rusqlite bundled 编译，避免 apt 镜像问题） ──
FROM rust:1 AS backend
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY crates/ ./crates/
RUN cargo build --release -p learnsys-api

# ── 运行 ──
FROM debian:bookworm-slim
WORKDIR /app
COPY --from=backend /app/target/release/learnsys-api /app/learnsys-api
COPY --from=frontend /app/frontend/dist /app/static

ENV LEARNSYS_STATIC_DIR=/app/static \
    LEARNSYS_BIND=0.0.0.0:7878 \
    RECALL_DB=/data/learnsys.db

VOLUME /data
EXPOSE 7878
CMD ["/app/learnsys-api"]
