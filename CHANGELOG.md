# Changelog

## v0.1.3

### Changed
- Limited concurrent Docker API requests to 5 simultaneous connections using a semaphore-based throttling system.

- Staggered stats requests across the polling interval instead of firing them all at once. Requests now spread evenly to avoid CPU spikes.

- Stats are now fetched only for containers visible in the current viewport, plus a small buffer. Containers scrolled out of view use cached data.

- Split container list updates (every 10s) from stats updates (configurable, default 3s) into separate background tasks.

- Added `--stats-interval` CLI flag to control stats polling frequency (1-10 seconds).

### Added
- CPU and memory usage now display directly in the container list view.

- Stale data indicator shows when container stats are older than 10 seconds.

- Viewport tracking ensures stats refresh as you scroll through the container list.

### Performance Improvements
- Reduced CPU usage by 60-80% on systems managing 20+ containers
- Eliminated periodic CPU spikes from simultaneous Docker API requests
- Improved responsiveness on single-core VPS instances
- Better scaling for systems with 50+ containers

## v0.1.2

### Changed
- Removed async overhead from UI rendering by switching to synchronous draw calls. The terminal no longer forces thread parking/unparking on every frame.

- Replaced `tokio::sync::RwLock` with `std::sync::RwLock` for shared state management. UI rendering is now a straightforward function call without async runtime overhead.

- Refactored all UI modules (`src/ui/*`) to operate synchronously. `block_in_place` workarounds in the event handler are no longer needed.

- Fixed lock contention between background stats collection and UI rendering. Write guards now properly drop before async sleep calls.

- Streamlined event loop by making `trigger_fetch` and `start_log_stream` synchronous. These functions now spawn background tasks without complicating the main loop logic.

### Performance Improvements
- Reduced CPU usage during idle and active states
- Eliminated thread contention between UI rendering and stats collection
- Improved frame timing consistency by removing async/await from the render path

## v0.1.1
- Initial public pre-1.0 release