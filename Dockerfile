# syntax=docker/dockerfile:1
FROM rust:latest as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM rust:latest
WORKDIR /app
COPY --from=builder /app/target/release/commitaura /usr/local/bin/commitaura
COPY README.md LICENSE /app/
COPY .env /app/.env
RUN apt-get update && apt-get install -y git && rm -rf /var/lib/apt/lists/*
ENV OPENAI_API_KEY=${OPENAI_API_KEY}
ENTRYPOINT ["commitaura"]
