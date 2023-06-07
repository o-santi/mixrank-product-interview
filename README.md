# Competitive matrix rendering

A competitive matrix renderer in the browser, built completely in rust.

## Usage

This package uses nix to manage installation of packages. It uses [`cargo-leptos`](https://github.com/leptos-rs/cargo-leptos) to manage building of both the backend server and the hydrated-wasm frontend. It is installed automatically from the nix configuration, so to run the server, you can run:
```sh
nix-shell
cargo leptos watch --release
```
