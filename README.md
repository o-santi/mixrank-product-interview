# Competitive matrix rendering

A competitive matrix renderer in the browser, built completely in rust.

## Usage

This package uses nix to manage installation of packages. It uses wasm-pack to build the wasm hydrated file. To build it:
```sh
nix-shell
wasm-pack build --target=web --debug --no-default-features --features=hydrate
cargo run --no-default-features --features=ssr --release
```
