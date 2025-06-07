# btrs

**btrs** (BitTorrent Rust Shell) is a terminal-based BitTorrent client written from scratch in Rust.

This is a personal project intended as a learning exercise. The goal is to explore networking, asynchronous programming, and protocol design in Rust. It is not intended for general use.

---

## Project Scope

The project focuses on implementing a minimal BitTorrent client with a terminal interface, without relying on existing BitTorrent libraries. This includes:

- Parsing `.torrent` files (BEP 3)
- Managing peer connections and protocol handshakes
- Downloading and verifying pieces
- Displaying state in a TUI

---

## Stack

- **Rust** — language and core tooling  
- **Tokio** — async runtime for networking and timers  
- **ratatui** — terminal user interface framework

---

## Status

Development is exploratory and ongoing. Currently focused on:

- Torrent metadata parsing
- TUI layout and rendering
- Initial peer protocol design

---

## License

This project is licensed under the [MIT License](LICENSE).
