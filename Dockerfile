FROM rust:1.65 as builder

WORKDIR /usr/src/capyhourly
COPY . .
RUN cargo install --path .


FROM debian:buster-slim

RUN apt update && \
    apt install openssl ca-certificates -y && \
    rm -rf /var/lib/apt/lists/*

COPY --from=builder /usr/local/cargo/bin/capyhourly /usr/local/bin/

CMD ["capyhourly"]