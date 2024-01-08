FROM rust:latest

WORKDIR /app
RUN apt update && apt install clang -y
COPY . .

ENV SQLX_OFFLINE true
ENV APP_ENVIRONMENT production

RUN cargo build --release

# When docker is executed. launch the binary!
ENTRYPOINT ["./target/release/zero2prod"]
