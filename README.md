# Dockyard 🚢

Dockyard is a terminal-based Docker container manager built specifically for resource-constrained VPS servers. It provides real-time insights into your containers with minimal overhead.

## Perfect For

* **Mcro VPS instances** (DigitalOcean, OVH Cloud, Linode, Vultr, etc)
* **Low-spec servers** (2–4 vCPU, 2–8 GB RAM)
* **Production environments** where every MB counts
* **Development servers** running multiple containers
* **Raspberry Pi** and other ARM-based servers

## Features ✨

* Written in Rust with async/await – no GIL, no garbage collection
* Concurrent stats fetching – query all containers in parallel
* Non-blocking UI updates – never freezes, always responsive
* Event-driven architecture with background workers

![Dockyard TUI](https://raw.githubusercontent.com/905timur/dockyard/main/tui.png)

## Installation 📦

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
| `↑`/`↓` or `j`/`k` | Navigate containers |
| `i` | View resource history graphs |
| `l` | View container logs |
| `r` | Restart container |
| `s` | Stop container |
| `t` | Start container |
| `d` | Remove container (force) |
| `f` | Toggle filter (all/running) |
| `q` | Quit |
