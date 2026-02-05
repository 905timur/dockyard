# Dockyard ⚓
![CI](https://github.com/905timur/dockyard/actions/workflows/ci.yml/badge.svg?branch=main)
![Release](https://img.shields.io/github/v/release/905timur/dockyard)
![License](https://img.shields.io/github/license/905timur/dockyard)
![Rust](https://img.shields.io/badge/rust-1.75%2B-blue)

Dockyard is a terminal-based Docker management tool built specifically for resource constrained VPS servers, written in Rust with performance in mind. It provides complete container and image lifecycle management with real-time monitoring, all with minimal overhead and an easy-on-the-eyes UI.

![Dockyard TUI](https://github.com/905timur/dockyard/blob/main/tui-screen.png)

## Perfect For
* **Low-spec servers** (1–4 vCPU, 512 MB–8 GB RAM)
* **Production environments** where every MB counts
* **Development servers** running multiple containers
* **Raspberry Pi** and other ARM-based servers

## Features 
- Written in Rust with async/await – no GIL, no garbage collection
- **Full image management** – pull, remove, inspect, and filter images
- **Interactive shell access** – drop into running containers with full TTY support
- **Container lifecycle controls** – start, stop, restart, pause, unpause, and remove
- Viewport-aware stats fetching – only queries containers visible on screen
- Staggered API requests – spreads Docker calls evenly to prevent CPU spikes
- Synchronous UI rendering – zero async overhead, no thread parking, instant frame updates
- Event-driven architecture with lock-free UI – background workers never block the terminal
- **Built-in Wiki**: Access project documentation and troubleshooting tips directly from the help menu (`?`)
- **Configurable polling intervals**: Tune performance for your environment

## Installation 

### Prerequisites
* Rust `1.70+` ([Install Rust](https://rustup.rs/))
* Docker daemon running
* Access to the Docker socket

### Install from Release (Recommended)
```bash
# Download the latest release (v0.3.1)
wget https://github.com/905timur/dockyard/releases/download/v0.3.1/dockyard-v0.3.1-x86_64.tar.gz

# Extract and install
tar -xzf dockyard-v0.3.1-x86_64.tar.gz
sudo mv dockyard /usr/local/bin/

# Run it
dockyard
