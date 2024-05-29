# PubTrust Chat

Chat, based of Public/Private key and signatures, and working on MQTT Protocol

![Showcase](showcase.gif)

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
- [ ] DirectMessages
- [ ] /q, /exit, /list, /help commands
- [ ] Better code structure (tbh looks like a 💩)
- [ ] Do something with thread channels, mutex, etc
- [ ] GitHub Actions
- [ ] Check windows support ~~(nobody cares)~~