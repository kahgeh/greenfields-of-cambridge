FROM rust:1.90 AS builder 
WORKDIR /app

COPY Cargo.toml Cargo.lock ./

RUN mkdir -p src 

COPY Cargo.toml ./

RUN echo "fn main() {}" > src/main.rs 

RUN cargo build --release --locked && \
    rm -rf src 

COPY . .

RUN touch src/main.rs 
RUN cargo build --release --locked

FROM debian:stable-slim as goc

WORKDIR /app

RUN set -eux; \
    apt-get update; \
    apt-get install -y --no-install-recommends ca-certificates; \
    rm -rf /var/lib/apt/lists/*; 


COPY --from=builder /app/target/release/greenfields-of-cambridge /usr/local/bin/
COPY --from=builder /app/static /app/static
EXPOSE 7100 

CMD ["greenfields-of-cambridge"]
