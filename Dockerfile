FROM rust:1.83.0-alpine AS builder
RUN apk add --no-cache \
    musl-dev \
    openssl-dev \
    pkgconfig
WORKDIR /usr/src/zfs-http-query
COPY . .
RUN cargo install --path .

FROM scratch
COPY --from=builder /usr/local/cargo/bin/external-dns-technitium-webhook /external-dns-technitium-webhook
CMD ["/external-dns-technitium-webhook"]
