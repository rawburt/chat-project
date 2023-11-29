# Chat Project

This project implements a custom client/server chat protocol. The protocol is described in the [PROTOCOL.md](./PROTOCOL.md) file.

## Usage

### Client

```sh
Usage: chat-client <ADDRESS>

Arguments:
  <ADDRESS>

Options:
  -h, --help     Print help
  -V, --version  Print version
```

Run via cargo:

```sh
cargo run --bin chat-client localhost:5456
```

Build and run executable:

```sh
cargo build --release
./target/release/chat-client localhost:5456
```

### Server

```sh
Usage: chat-server <ADDRESS>

Arguments:
  <ADDRESS>

Options:
  -h, --help     Print help
  -V, --version  Print version
```

Run via cargo:

```sh
RUST_LOG=info cargo run --bin chat-server localhost:545
```

Running with `RUST_LOG=info` enables logging to STDOUT.

Build and run executable:

```sh
cargo build --release
RUST_LOG=info ./target/release/chat-server localhost:5456
```

## References

* "Programming Rust" by Jim Blandy, Jason Orendorff, and Leonora F. S. Tindall
* https://github.com/ProgrammingRust/async-chat/tree/master
* https://tokio.rs/
* https://www.youtube.com/watch?v=Iapc-qGTEBQ
* https://github.com/tokio-rs/tokio/blob/master/examples/
* https://github.com/matszpk/simple-irc-server/tree/main
