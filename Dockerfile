FROM lukemathwalker/cargo-chef:latest-rust-1 AS chef
WORKDIR /app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder 
COPY --from=planner /app/recipe.json recipe.json
# Build dependencies - this is the caching Docker layer!
RUN cargo chef cook --release --recipe-path recipe.json
# Build application
COPY . .
RUN cargo build --release

# We do not need the Rust toolchain to run the binary!
FROM debian:bookworm-slim AS build2
RUN apt update && apt install -y imagemagick webp libjxl-tools curl
WORKDIR /app
COPY ./assets assets
COPY ./content content
COPY --from=builder /app/target/release/holligerme /usr/local/bin
RUN /usr/local/bin/holligerme --build


FROM debian:bookworm-slim AS runtime
WORKDIR /app
COPY ./assets assets
COPY --from=builder /app/target/release/holligerme /usr/local/bin
COPY --from=build2 /app/database database
ENTRYPOINT ["/usr/local/bin/holligerme"]
