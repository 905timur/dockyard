# Dockyard ⚓
![CI](https://github.com/905timur/dockyard/actions/workflows/ci.yml/badge.svg?branch=main)
![Release](https://img.shields.io/github/v/release/905timur/dockyard)
![License](https://img.shields.io/github/license/905timur/dockyard)
![Rust](https://img.shields.io/badge/rust-1.75%2B-blue)

Dockyard is a terminal-based Docker management tool built specifically for resource constrained VPS servers, written in Rust with performance in mind. It provides complete container and image lifecycle management with real-time monitoring, all with minimal overhead and an easy-on-the-eyes UI.

![Dockyard TUI](https://github.com/905timur/dockyard/blob/main/screen.png)

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
- Configurable polling intervals – tune performance for your environment

## Installation 

### Prerequisites
* Rust `1.70+` ([Install Rust](https://rustup.rs/))
* Docker daemon running
* Access to the Docker socket

### Option 1: Install from Release (Recommended)
```bash
# Download the latest release (v0.2.1)
wget https://github.com/905timur/dockyard/releases/download/v0.2.0/dockyard-v0.2.1-x86_64.tar.gz

# Extract and install
tar -xzf dockyard-v0.2.1-x86_64.tar.gz
sudo mv dockyard /usr/local/bin/

# Run it
dockyard
```

### Option 2: Build from Source
```bash
# Clone the repository
git clone https://github.com/905timur/dockyard.git
cd dockyard

# Build and run (release mode for best performance)
cargo run --release
```

### Option 3: Install with Cargo
```bash
cargo install --git https://github.com/905timur/dockyard.git --tag v0.2.1
dockyard
```

### Ensure Docker permissions are set
```bash
sudo usermod -aG docker $USER
# Log out and log back in for changes to take effect
```

## Usage
```bash
# Run with default settings (3 second stats interval)
dockyard

# Adjust stats polling interval (1-10 seconds)
dockyard --stats-interval 5
```

## Navigation

### Global Keys
| Key | Action |
|-----|--------|
| `?` | Help menu |
| `Tab` | Switch between Containers and Images views |
| `q` | Quit |

### Container View
| Key | Action |
|-----|--------|
| `↑↓` or `jk` | Navigate containers |
| `i` | View resource history graphs |
| `l` | View container logs |
| `e` | Launch interactive shell (`/bin/bash` or `/bin/sh`) |
| `r` | Restart container |
| `s` | Stop container |
| `t` | Start container |
| `p` | Pause container |
| `u` | Unpause container |
| `d` | Remove container (force) |
| `f` | Toggle filter (all/running) |

### Image View
| Key | Action |
|-----|--------|
| `↑↓` or `jk` | Navigate images |
| `Enter` or `Space` | Inspect image details |
| `s` | Toggle sort (Creation Date ▲/▼, Size ▲/▼) |
| `f` | Toggle dangling image filter |
| `p` | Pull new image |
| `d` | Remove image |
| `D` | Force remove image |

## Performance Notes
- Auto-refreshes container list every 10 seconds
- Auto-refreshes image list every 30 seconds
- Stats update interval configurable via `--stats-interval` flag (default: 3 seconds)
- Only fetches stats for containers in the current viewport
- Concurrent API requests limited to 5 simultaneous connections