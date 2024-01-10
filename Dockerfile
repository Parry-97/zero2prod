FROM clux/muslrust:stable AS chef
USER root
RUN cargo install cargo-chef
WORKDIR /app

FROM chef AS planner
COPY . .
RUN cargo chef prepare  --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
# Build dependencies - this is the caching Docker layer!
RUN cargo chef cook --release --recipe-path recipe.json

RUN rustup target add x86_64-unknown-linux-musl

WORKDIR /app
COPY . .

ENV SQLX_OFFLINE true

RUN cargo build --target x86_64-unknown-linux-musl --release

#NOTE: It seems alpine images are smaller than googles distroless images
FROM alpine AS runtime
# FROM gcr.io/distroless/cc as runtime
COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/zero2prod /
COPY --from=builder /app/configuration configuration
ENV APP_ENVIRONMENT production
CMD ["./zero2prod"]
