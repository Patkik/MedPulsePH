FROM rust:1.90-slim-bookworm AS build
RUN apt-get update && apt-get install -y --no-install-recommends build-essential ca-certificates \
    && rm -rf /var/lib/apt/lists/*
WORKDIR /workspace
COPY Cargo.toml Cargo.lock ./
COPY backend/Cargo.toml backend/Cargo.toml
COPY backend/src backend/src
RUN cargo build --locked --release -p medpulse-api

FROM build AS tools
RUN rustup component add clippy

FROM debian:bookworm-slim
RUN groupadd --system medpulse && useradd --system --gid medpulse medpulse
COPY --from=build /workspace/target/release/medpulse-api /usr/local/bin/medpulse-api
USER medpulse
EXPOSE 8080
CMD ["medpulse-api"]
