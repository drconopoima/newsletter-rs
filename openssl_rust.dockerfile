FROM docker.io/library/rust:1.59-slim-bullseye

RUN apt-get update -y \
    && apt-get install -y --no-install-recommends pkg-config libssl-dev openssl ca-certificates \
    # Clean up
    && apt-get autoremove -y \
    && apt-get clean -y \
    && rm -rf /var/lib/apt/lists/*
