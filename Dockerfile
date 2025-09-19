FROM rust:1.90.0-alpine AS builder
RUN apk add --no-cache \
    musl-dev \
    openssl-libs-static openssl-dev \
    pkgconfig
WORKDIR /usr/src/external-dns-technitium-webhook
COPY . .
RUN cargo install --path .

FROM scratch
COPY --from=builder /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/ca-certificates.crt
COPY --from=builder /usr/local/cargo/bin/external-dns-technitium-webhook /external-dns-technitium-webhook
CMD ["/external-dns-technitium-webhook"]
