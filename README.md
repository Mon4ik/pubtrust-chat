# PubTrust Chat

Chat, based of Public/Private key and signatures, and working on MQTT Protocol

![Showcase](docs/showcase.gif)

> [!CAUTION]
>
> Poor alpha version, may contain crashes, memory leak, skill issues, zero-day exploits, and other basic bugs

## Installation

1. Install Rust
2. Install OpenSSL [(more info on docs.rs)](https://docs.rs/openssl/latest/openssl/#automatic)
3. Clone
   ```shell
   git clone https://...
   ```
4. Build && run
   ```shell
   # 1 variant: debug run
   cargo run -- [options]
   
   # 2 variant: debug build
   cargo build
   ./target/debug/pubtrust-chat
   
   # 3 variant: release build
   cargo build -r
   ./target/debug/pubtrust-chat
   ```

## TODO

- [X] Better code structure (tbh looks like a 💩)
- [X] Do something with thread channels, mutex, etc
- [ ] GitHub Actions
- [ ] Make functionality for /list, /room 
- [ ] Direct Messages
- [ ] Check windows support ~~(nobody cares)~~