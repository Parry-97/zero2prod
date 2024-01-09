FROM rust:1 AS chef 
# We only pay the installation cost once, 
# it will be cached from the second build onwards
RUN cargo install cargo-chef 
WORKDIR app

FROM chef AS planner
COPY . .
RUN cargo chef prepare  --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
# Build dependencies - this is the caching Docker layer!
RUN cargo chef cook --release --recipe-path recipe.json

RUN apt-get update && apt-get install musl-tools -y
RUN rustup target add x86_64-unknown-linux-musl

WORKDIR /app
COPY . .

ENV SQLX_OFFLINE true

RUN cargo build --target x86_64-unknown-linux-musl --release

FROM gcr.io/distroless/cc as runtime
COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/zero2prod /
COPY --from=builder /app/configuration configuration
ENV APP_ENVIRONMENT production
CMD ["./zero2prod"]


# When docker is executed. launch the binary!
# ENTRYPOINT ["./target/release/zero2prod"]
