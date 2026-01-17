# Changelog

## v0.2.0

### Added
- **Image Management**: New image list view accessible via `Tab` key
  - Displays repository, tag, image ID, size, and creation date
  - Auto-refreshes every 30 seconds
  - Sort images by creation date or size (ascending/descending) using `s` key
  - Visual sort indicators (▲/▼) in table headers
  - Filter dangling images using `f` key
  - Navigate with `j`/`k` or arrow keys

- **Pull Images**: Press `p` in image view to pull new images
  - Modal dialog for entering image name (e.g., `nginx:latest`)
  - Real-time progress display streamed to contextual output pane
  - Background task processing to prevent UI freezing

- **Remove Images**: Delete images with `d` key (normal) or `D` key (forced removal)
  - Confirmation modal before removal
  - Graceful error handling

- **Image Details**: Press `Enter` or `Space` on selected image to view full details
  - Displays ID, tags, architecture, OS, size, environment variables, labels, and exposed ports
  - Details shown in left pane (contextual details)
  - Automatically fetched on selection change

- **Container Pause/Unpause**: Pause running containers with `p` key, unpause with `u` key
  - Visual feedback shows paused state with `‖` indicator in yellow
  - Automatic state validation (only pause running, only unpause paused)
  - Automatic list refresh after operation

- **Interactive Shell Access**: Press `e` to launch interactive shell in running containers
  - Suspends TUI and launches `docker exec -it` with full TTY support
  - Attempts `/bin/bash` first, falls back to `/bin/sh` if unavailable
  - Proper signal handling and terminal control
  - Restores TUI automatically on shell exit

- **Status Bar**: Bottom bar displays available keybindings for current view
  - Context-sensitive help for containers vs images view
  - Always visible for quick reference

### Changed
- Restructured app to support multiple views (Containers/Images) with view-specific keybindings
- Rewrote key handling system to support modals and context-sensitive shortcuts
- Updated event loop to handle view-specific logic and shell suspension/restoration
- Implemented fixed four-pane layout system
  - Left pane: Contextual details (container/image details)
  - Top right pane: Main list (containers/images)
  - Bottom right pane: Contextual output (logs/pull progress)
  - Modals: Overlay center when active

### Technical
- Added `ImageInfo` struct in `src/types.rs`
- Created `src/docker/images.rs` for image API operations (list, pull, remove, inspect, prune)
- Created `src/docker/exec.rs` for interactive shell execution using `std::process::Command`
- Added `src/ui/image_list.rs` and `src/ui/image_details.rs` UI components
- Enhanced `src/app.rs` with image state management and background refresh tasks
- Modified `src/events/handler.rs` to support TUI suspension/restoration for exec operations
- Updated `src/events/key_bindings.rs` with view-aware key handling

## v0.1.3

### Changed
- Limited concurrent Docker API requests to 5 simultaneous connections using a semaphore-based throttling system
- Staggered stats requests across the polling interval instead of firing them all at once. Requests now spread evenly to avoid CPU spikes
- Stats are now fetched only for containers visible in the current viewport, plus a small buffer. Containers scrolled out of view use cached data
- Split container list updates (every 10s) from stats updates (configurable, default 3s) into separate background tasks
- Added `--stats-interval` CLI flag to control stats polling frequency (1-10 seconds)

### Added
- CPU and memory usage now display directly in the container list view
- Stale data indicator shows when container stats are older than 10 seconds
- Viewport tracking ensures stats refresh as you scroll through the container list

### Performance Improvements
- Reduced CPU usage by 60-80% on systems managing 20+ containers
- Eliminated periodic CPU spikes from simultaneous Docker API requests
- Improved responsiveness on single-core VPS instances
- Better scaling for systems with 50+ containers

## v0.1.2

### Changed
- Removed async overhead from UI rendering by switching to synchronous draw calls. The terminal no longer forces thread parking/unparking on every frame
- Replaced `tokio::sync::RwLock` with `std::sync::RwLock` for shared state management. UI rendering is now a straightforward function call without async runtime overhead
- Refactored all UI modules (`src/ui/*`) to operate synchronously. `block_in_place` workarounds in the event handler are no longer needed
- Fixed lock contention between background stats collection and UI rendering. Write guards now properly drop before async sleep calls
- Streamlined event loop by making `trigger_fetch` and `start_log_stream` synchronous. These functions now spawn background tasks without complicating the main loop logic

### Performance Improvements
- Reduced CPU usage during idle and active states
- Eliminated thread contention between UI rendering and stats collection
- Improved frame timing consistency by removing async/await from the render path

## v0.1.1
- Initial public pre-1.0 release