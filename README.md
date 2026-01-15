# Dockyard ⚓
![CI](https://github.com/905timur/dockyard/actions/workflows/ci.yml/badge.svg?branch=main)
![Release](https://img.shields.io/github/v/release/905timur/dockyard)
![License](https://img.shields.io/github/license/905timur/dockyard)
![Rust](https://img.shields.io/badge/rust-1.75%2B-blue)

Dockyard is a terminal-based Docker container manager built specifically for resource constrained VPS servers, written in Rust and with performance in mind. 
It provides real-time insights into your containers with minimal overhead and an easy-on-the-eyes UI.

![Dockyard TUI](https://github.com/905timur/dockyard/blob/main/screen.png)

## Perfect For

* **Low-spec servers** (2–4 vCPU, 2–8 GB RAM)
* **Production environments** where every MB counts
* **Development servers** running multiple containers
* **Raspberry Pi** and other ARM-based servers

## Features 

- Written in Rust with async/await – no GIL, no garbage collection
- Concurrent stats fetching – query all containers in parallel with async background workers
- Synchronous UI rendering – zero async overhead, no thread parking, instant frame updates
- Event-driven architecture with lock-free UI – background workers never block the terminal

## Installation 

### Prerequisites

* Rust `1.70+` ([Install Rust](https://rustup.rs/))
* Docker daemon running
* Access to the Docker socket

### Quick Start

```bash
# Clone the repository
git clone https://github.com/905timur/dockyard.git
cd dockyard

# Build and run (release mode for best performance)
cargo run --release
```

**OR**

### System-wide Install

```bash
cargo install --path .
dockyard
```

### Ensure credentials are set

```bash
sudo usermod -aG docker $USER
# Log out and log back in
```

## Navigation

| Key | Action |
|-----|--------|
| `?` | Help menu |
| `↑`/`↓` or `j`/`k` | Navigate containers |
| `i` | View resource history graphs |
| `l` | View container logs |
| `r` | Restart container |
| `s` | Stop container |
| `t` | Start container |
| `d` | Remove container (force) |
| `f` | Toggle filter (all/running) |
| `q` | Quit |
