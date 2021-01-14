FROM rust:latest as builder

RUN cargo install cargo-make wasm-pack

WORKDIR /usr/src/frontend
COPY . /usr/src/frontend

RUN --mount=type=cache,mode=0755,target=/usr/local/cargo/registry \
    --mount=type=cache,mode=0755,target=/usr/src/cdl/target \
    cargo make pack

FROM nginx:alpine

COPY --from=builder /usr/src/frontend/www /usr/share/nginx/html
