FROM docker.io/library/rust:1.59-slim-bullseye as build

RUN USER=root cargo new --bin newsletter-rs

WORKDIR /newsletter-rs

RUN touch ./src/lib.rs

# Copy manifests
COPY ./Cargo.toml ./Cargo.lock ./

# Cache dependencies
RUN cargo build --release

RUN rm ./src/*.rs ./target/release/deps/newsletter_rs* ./target/release/deps/libnewsletter_rs* ./target/release/libnewsletter_rs* ./target/release/newsletter-rs*

COPY ./src ./src

# Build for release
RUN cargo build --release

FROM docker.io/debian:buster-slim

# Copy build artifact
COPY --from=build /newsletter-rs/target/release/newsletter-rs .

COPY ./configuration.yaml ./

# Startup command
CMD ["./newsletter-rs"]
