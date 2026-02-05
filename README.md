# Dockyard ⚓

![CI](https://github.com/905timur/dockyard/actions/workflows/ci.yml/badge.svg?branch=main)
![Release](https://img.shields.io/github/v/release/905timur/dockyard)
![License](https://img.shields.io/github/license/905timur/dockyard)
![Rust](https://img.shields.io/badge/rust-1.75%2B-blue)

Dockyard is a **terminal-based Docker management tool** built specifically for **resource-constrained VPS servers**.  
Written in **Rust** with performance as a first-class concern, it provides complete container and image lifecycle management with real-time monitoring—all with minimal overhead and an easy-on-the-eyes TUI.

![Dockyard TUI](https://github.com/905timur/dockyard/blob/main/tui-screen.png)

---

## Perfect For

- **Low-spec servers** (1–4 vCPU, 512 MB–8 GB RAM)
- **Production environments** where every MB counts
- **Development servers** running multiple containers
- **Raspberry Pi** and other ARM-based systems

---

## Features

- Written in **Rust** with `async/await` — no GIL, no garbage collection
- **Full image management** — pull, remove, inspect, and filter images
- **Container lifecycle controls** — start, stop, restart, pause, unpause, remove
- **Interactive shell access** — exec into running containers with full TTY support
- Viewport-aware stats fetching — only queries containers visible on screen
- Staggered Docker API requests — avoids CPU spikes on small systems
- Synchronous UI rendering — zero async overhead, instant frame updates
- Event-driven architecture with a **lock-free UI**
- **Built-in Wiki** — access documentation and troubleshooting via `?`
- **Configurable polling intervals** — tune performance for your environment

---

## Installation

### Prerequisites

- Rust **1.75+** ([Install Rust](https://rustup.rs/))
- Docker daemon running
- Access to the Docker socket

---

### Option 1: Install from Release (Recommended)

```bash
# Download the latest release (v0.3.1)
wget https://github.com/905timur/dockyard/releases/download/v0.3.1/dockyard-v0.3.1-x86_64.tar.gz

# Extract and install
tar -xzf dockyard-v0.3.1-x86_64.tar.gz
sudo mv dockyard /usr/local/bin/

# Run
dockyard
```

### Option 2: Install from Repo
```bash
# Clone the repository
git clone https://github.com/905timur/dockyard.git
cd dockyard

# Build and run (release mode for best performance)
cargo run --release
```
### Option 3: Install with Cargo
```bash
cargo install --git https://github.com/905timur/dockyard.git --tag v0.3.1
dockyard
```

## Docker Permissions

Ensure your user can access the Docker socket:
```bash
sudo usermod -aG docker $USER
# Log out and log back in for changes to take effect
```

## Quick Start
```bash
# Run with default settings (3-second stats interval)
dockyard

# Adjust stats polling interval (1–10 seconds)
dockyard --stats-interval 5
```
## Usage

See the repo wiki page or help menu inside the application.
