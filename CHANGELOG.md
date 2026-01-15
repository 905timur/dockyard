# Changelog

## v0.1.1
- Initial public pre-1.0 release

## v0.1.2
## Changed
- Removed async overhead from UI rendering by switching to synchronous draw calls. The terminal no longer forces thread parking/unparking on every frame, eliminating unnecessary context switches.

- Replaced `tokio::sync::RwLock` with `std::sync::RwLock` for shared state management. UI rendering is now a straightforward function call without async runtime overhead.

- Refactored all UI modules (`src/ui/*`) to operate synchronously, removing the need for `block_in_place` workarounds in the event handler.

- Fixed lock contention between background stats collection and UI rendering. Write guards now properly drop before async sleep calls, preventing the UI from blocking on locks held by background tasks.

- Streamlined event loop by making `trigger_fetch` and `start_log_stream` synchronous. These functions now simply spawn background tasks without complicating the main loop logic.

## Performance Improvements

- Reduced CPU usage during idle and active states
- Eliminated thread contention between UI rendering and stats collection
- Improved frame timing consistency by removing async/await from the render path