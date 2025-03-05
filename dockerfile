FROM rust:1.73-slim-bullseye AS build

RUN apt-get update && \
    apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY Cargo.toml Cargo.lock ./
COPY src/ src/

RUN cargo build --release

FROM debian:bullseye-slim

RUN apt-get update && \
    apt-get install -y \
    openssl \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/* 

COPY --from=build /app/target/release/cloud-native-proxy /usr/local/bin/

EXPOSE 80 443 3000

CMD ["/usr/local/bin/cloud-native-proxy"]