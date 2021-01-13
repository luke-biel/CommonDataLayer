FROM rust:latest as builder

RUN cargo install cargo-make wasm-pack

WORKDIR /usr/src/mnemosyne
COPY . /usr/src/mnemosyne

RUN --mount=type=cache,mode=0755,target=/usr/local/cargo/registry \
    --mount=type=cache,mode=0755,target=/usr/src/cdl/target \
    cargo make rebuild

FROM nginx:alpine

COPY --from=builder /usr/src/mnemosyne/www /usr/share/nginx/html
