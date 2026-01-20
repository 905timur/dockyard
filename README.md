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
wget https://github.com/905timur/dockyard/releases/download/v0.2.1/dockyard-v0.2.1-x86_64.tar.gz

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

## Quick Start Guide

### Managing Containers
Use the `Tab` key to switch between container and image views.

When you first launch Dockyard, you'll see your container list. Use `j`/`k` or arrow keys to navigate up and down. The interface shows you each container's name, image, status, ports, and real-time CPU/memory usage.

Press `Enter` on any container to see detailed information in the left pane, including environment variables, volumes, networks, and labels. Press `l` to stream logs in real-time, or `e` to drop into an interactive shell inside the container (Dockyard will suspend the TUI and hand control to your shell, then restore everything when you exit).

You can control containers with `s` (stop), `t` (start), `r` (restart), `p` (pause), `u` (unpause), and `d` (force remove). Press `f` to toggle between viewing all containers or just running ones.

### Managing Images
Press `Shift+Tab` to switch to the image view. Here you'll see all Docker images on your system with their repository names, tags, IDs, sizes, and creation dates. The list auto-refreshes every 30 seconds.

Navigate with `j`/`k` or arrow keys, then press `Enter` or `Space` to inspect any image's full details in the left pane. You can sort images by pressing `s` (cycles through creation date ascending/descending and size ascending/descending) or filter dangling images with `f`.

### Pulling New Images
To download a new image from a registry like Docker Hub, press `p` while in the image view. A dialog will appear asking for the image name. Type something like `nginx:latest`, `postgres:15`, `redis:alpine`, or `ubuntu:22.04` and hit Enter. 

Dockyard will start pulling the image and stream the download progress in real-time in the bottom-right pane. You'll see each layer being downloaded just like running `docker pull` from the command line, but the UI stays responsive so you can navigate around and check other things while it downloads. When the pull completes, the image list automatically refreshes and your new image appears.

This is useful when you want to run a new service (like pulling `nginx` to set up a web server), test a different version of something (pulling `node:20` for the latest Node.js), or prepare images before creating containers from them.

### Removing Images
Select an image and press `d` to remove it (you'll get a confirmation prompt). If the image is in use by containers, you can force removal with `D` (Shift+d), though this will also prompt for confirmation to prevent accidents.

## Navigation Reference

### Global Keys
| Key | Action |
|-----|--------|
| `?` | Help menu |
| `Tab` or `Shift+Tab` | Switch between Containers and Images views |
| `q` | Quit |

### Container View
| Key | Action |
|-----|--------|
| `↑↓` or `jk` | Navigate containers |
| `Enter` or `Space` | View detailed container info |
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
| `p` | Pull new image from registry |
| `d` | Remove image |
| `D` | Force remove image |

## Status Indicators

### Container States
- `▶` (green) – Running
- `■` (red) – Stopped
- `‖` (yellow) – Paused

### Visual Feedback
- Sort indicators (`▲`/`▼`) appear in table headers showing current sort direction
- Stats marked as `(stale)` are older than 10 seconds
- Real-time progress bars show ongoing operations like image pulls
- Confirmation prompts appear for destructive actions

## Performance Notes
- Auto-refreshes container list every 10 seconds
- Auto-refreshes image list every 30 seconds
- Stats update interval configurable via `--stats-interval` flag (default: 3 seconds)
- Only fetches stats for containers in the current viewport (visible on screen)
- Concurrent API requests limited to 5 simultaneous connections
- Requests are staggered across the polling interval to prevent CPU spikes

## Troubleshooting

### Permission Denied
If you get permission errors connecting to Docker, make sure your user is in the `docker` group:
```bash
sudo usermod -aG docker $USER
```
Then log out and back in for the changes to take effect.

### High CPU Usage
If you notice high CPU usage, try increasing the stats interval:
```bash
dockyard --stats-interval 5
```
This reduces how often Dockyard polls Docker for container stats.

### Container Shell Not Working
The interactive shell feature (`e` key) requires that containers have either `/bin/bash` or `/bin/sh` available. Some minimal container images might not include these shells.

## License
Dockyard is dual-licensed under MIT and Apache 2.0.

## Acknowledgments
Built with [Ratatui](https://github.com/ratatui-org/ratatui) for the TUI and [Bollard](https://github.com/fussybeaver/bollard) for Docker API integration.