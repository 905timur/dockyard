- Fork the repo
- Create a feature branch
- Run `cargo check` and `cargo test`
- Open a PR

# Good First Issues for Dockyard
## Issue 1 — Help popup: add missing Image actions and rows to the Help table

**Problem**
The help popup (src/ui/help.rs) currently has a truncated Image Actions section. New contributors and users rely on this popup to learn keybindings. The Image Actions section should list the image-specific keys (pull, remove, details, and sort hints) to be complete and consistent with the status bar and UI.

**Acceptance criteria**
- [ ] Add Image Actions rows to `src/ui/help.rs` so the popup shows at least:
  - `p` — Pull
  - `d` — Remove Image
  - `Enter` — Details
  - Sort toggle hints for Size/Created (as short informative rows)
- [ ] Use the existing table/Row style (Row::new(vec![...])) to match other sections.
- [ ] `cargo check` passes (no Docker runtime required).
- [ ] The PR description includes a short note how to verify locally (run `cargo run` and press `?`).

**Relevant files / modules**
- `src/ui/help.rs`

**Preferred Issue Category**
- Documentation / TUI / UX polish

---

## Issue 2 — Add unit tests for `format_bytes` in container list

**Problem**
`format_bytes` in `src/ui/container_list.rs` formats byte sizes for display. Small formatting or rounding regressions are easy to introduce. Focused unit tests will prevent accidental changes and help new contributors understand the UI formatting logic.

**Acceptance criteria**
- [ ] Add a `#[cfg(test)]` test module inside `src/ui/container_list.rs` with tests asserting current behavior for three representative values:
  - ~1 KB (e.g., `1_024`) -> expected string according to current implementation
  - ~1 MB (e.g., `1_048_576`) -> expected string
  - ~1 GB (e.g., `1_073_741_824`) -> expected string
- [ ] Tests are self-contained and run with `cargo test`.
- [ ] Add a one-line comment above the tests explaining the purpose (prevent regressions in UI size formatting).

**Relevant files / modules**
- `src/ui/container_list.rs`

**Preferred Issue Category**
- Tests

---

## Issue 3 — Add unit tests for `format_time` in image list

**Problem**
`format_time` in `src/ui/image_list.rs` converts UNIX timestamps to relative strings ("Xd ago", "Xh ago", "Xm ago"). This helper is easy to break. Small tests will ensure consistent human-readable timestamps.

**Acceptance criteria**
- [ ] Add a `#[cfg(test)]` test module inside `src/ui/image_list.rs` with tests for:
  - A timestamp a few minutes ago -> `"Xm ago"`
  - A timestamp a few hours ago -> `"Xh ago"`
  - A timestamp several days ago -> `"Xd ago"`
- [ ] Use `chrono` helpers in tests to construct deterministic timestamps (documented in the test).
- [ ] Tests run with `cargo test`.

**Relevant files / modules**
- `src/ui/image_list.rs`

**Preferred Issue Category**
- Tests

---

## Issue 4 — CONTRIBUTING.md: add Developer Experience (DX) section for local dev & tests

**Problem**
`CONTRIBUTING.md` currently lists only minimal steps. New contributors would benefit from a short DX section explaining how to run tests, run the app locally for UI checks (without requiring Docker for build/test), and formatting/linting expectations.

**Acceptance criteria**
- [ ] Update only `CONTRIBUTING.md` by adding a concise "Developer experience" section (≤ 12 lines) that includes:
  - Commands to run tests: `cargo test`
  - Commands to build/run locally: `cargo run --release` (and note building doesn't require Docker; runtime Docker is required to exercise Docker features)
  - Formatting guidance: `cargo fmt` and `cargo fmt -- --check` (ask contributors to run `cargo fmt` before opening a PR)
  - Optional lint suggestion: `cargo clippy` (as an optional step, not required)
- [ ] Keep the change file-scoped (only `CONTRIBUTING.md` modified).
- [ ] Add a short one-line note clarifying Docker is only required to exercise runtime Docker functionality, not to compile or run unit tests.

**Relevant files / modules**
- `CONTRIBUTING.md`

**Preferred Issue Category**
- Developer experience (DX) / Documentation

---

## Issue 5 — TUI: render "debug" log lines in dim/gray style in logs renderer

**Problem**
`src/ui/logs.rs` colors log lines based on substrings "error", "warn", "info". "debug" lines are not handled specially and are displayed as white. Rendering debug lines in a dim/gray style improves visual scanning of important logs.

**Acceptance criteria**
- [ ] Modify `src/ui/logs.rs` to detect the substring `"debug"` (case-insensitive) and style those lines using `Color::DarkGray` (or similar dim style consistent with the project).
- [ ] The change touches only `src/ui/logs.rs`.
- [ ] `cargo check` passes.
- [ ] The PR description notes how to manually verify (run the app and view logs showing debug lines).

**Relevant files / modules**
- `src/ui/logs.rs`

**Preferred Issue Category**
- TUI / UX polish

Which would you like next?
