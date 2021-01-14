# Mnemosyne

...is a frontend application with intention to provide admin panel to whole CDL infrastructure.

## Setup

### Requirements

* cargo-make
* wasm-pack
* npm (for `http-server` - `npm install http-server -g`)
* `api` and `schema-registry` instances running on localhost

### Running

Run command `cargo make start` in `frontend/` directory.

## Docker deployment

Mnemosyne can be run via docker image located in it's directory. It sets up Nginx instance serving www files.
