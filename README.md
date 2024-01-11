# monithor


## Development

Start with the following command

> RUST_LOG=info cargo run --bin web


Watch changes

> RUST_LOG=info cargo watch -w src -w static -x 'run --bin web'

## Build

> cargo build --release

Use the `web` binary

## Usage

Access UI at http://localhost:8899

Socket to receive session information is at port 7878




