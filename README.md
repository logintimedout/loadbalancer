# Rust Async Load Balancer

A simple asynchronous HTTP load balancer built with Rust, Tokio, Axum, and Hyper. It runs two backend servers and a round-robin load balancer in a single binary.

> **Note:** This is a learning project and made this in English Class so the code isn't great, also the backend configuration is hardcoded so you need to recompile it,
might use serde to make a proper config file in JSON

## Features

- Round-robin request distribution across two backends
- Built-in backend servers on ports `8081` and `8082`
- Load balancer listener on port `3000`
- Async request forwarding using Hyper and Tokio

## How It Works

1. The binary starts two Axum backend servers, each responding with `Welcome to Backend Server N`.
2. The load balancer listens on `0.0.0.0:3000` and accepts incoming HTTP requests.
3. Each request is forwarded to the next backend in round-robin order using Hyper's HTTP/1 client.

## Running

```bash
cd load_balancer
cargo run
```

Then test with curl:

```bash
curl http://127.0.0.1:3000/
```

Requests will alternate between backend 1 (`8081`) and backend 2 (`8082`).

## Configuration

Ports are hardcoded in `src/main.rs`:

```rust
const BACKEND_1_ADDR: &str = "127.0.0.1:8081";
const BACKEND_2_ADDR: &str = "127.0.0.1:8082";
const LISTENER_ADDR: &str = "0.0.0.0:3000";
```

## Dependencies

- [tokio](https://tokio.rs) — async runtime
- [axum](https://github.com/tokio-rs/axum) — backend HTTP server
- [hyper](https://hyper.rs) — HTTP client/server
- [hyper-util](https://github.com/hyperium/hyper-util) — Tokio integration for Hyper
