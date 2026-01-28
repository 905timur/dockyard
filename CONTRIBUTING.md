# Contributing 

## Getting started

1. Fork the repository.
2. Create a feature branch for your change.
3. Run `cargo check` and `cargo test` to make sure everything passes.
4. Open a pull request with a short description of what you changed and how to verify it.

---

## Good first issues

The issues below are intentionally scoped to be approachable for new contributors.

### Issue 1: Help popup is missing Image actions

**Problem**  
The help popup in `src/ui/help.rs` has an incomplete Image Actions section. The help screen is one of the main ways users discover keybindings, so this section should fully reflect the image related actions shown in the UI.

**What to do**
- Add Image Actions rows to `src/ui/help.rs` so the popup shows at least:
  - `p` for Pull
  - `d` for Remove Image
  - `Enter` for Details
  - Short rows describing sort toggles for Size and Created
- Use the existing table and row style, for example `Row::new(vec![...])`, to match other sections.

**How to verify**
- Run `cargo run`.
- Press `?` to open the help popup and confirm the Image Actions section is complete.

**Relevant files**
- `src/ui/help.rs`

---

### Issue 2: Add unit tests for `format_bytes` in the container list

**Problem**  
The `format_bytes` helper in `src/ui/container_list.rs` formats byte sizes for display. Small changes can accidentally alter the output. Unit tests help lock in the current behavior and make future changes safer.

**What to do**
- Add a `#[cfg(test)]` test module inside `src/ui/container_list.rs`.
- Add tests that assert the current output for:
  - About 1 KB, for example `1_024`
  - About 1 MB, for example `1_048_576`
  - About 1 GB, for example `1_073_741_824`
- Add a one line comment explaining that the tests exist to prevent UI formatting regressions.

**How to verify**
- Run `cargo test` and confirm all tests pass.

**Relevant files**
- `src/ui/container_list.rs`

---

### Issue 3: Add unit tests for `format_time` in the image list

**Problem**  
The `format_time` helper in `src/ui/image_list.rs` converts UNIX timestamps into relative strings like `Xm ago`, `Xh ago`, and `Xd ago`. This logic is easy to break without noticing.

**What to do**
- Add a `#[cfg(test)]` test module inside `src/ui/image_list.rs`.
- Add tests that cover:
  - A timestamp a few minutes ago, expecting `Xm ago`
  - A timestamp a few hours ago, expecting `Xh ago`
  - A timestamp several days ago, expecting `Xd ago`
- Use `chrono` helpers to create deterministic timestamps and mention this in a short comment.

**How to verify**
- Run `cargo test`.

**Relevant files**
- `src/ui/image_list.rs`

---

### Issue 4: Improve CONTRIBUTING.md with a Developer Experience section

**Problem**  
`CONTRIBUTING.md` currently contains only minimal setup steps. A short Developer Experience section would make it easier for new contributors to get productive quickly.

**What to do**
- Update only `CONTRIBUTING.md`.
- Add a concise "Developer experience" section, no more than 12 lines, that includes:
  - How to run tests with `cargo test`
  - How to build and run locally with `cargo run --release`
  - A note that Docker is not required to build or run tests, but is required to exercise Docker functionality at runtime
  - Formatting guidance using `cargo fmt` and `cargo fmt -- --check`
  - An optional linting step using `cargo clippy`

**How to verify**
- Review the updated `CONTRIBUTING.md` and confirm the instructions are clear and concise.

**Relevant files**
- `CONTRIBUTING.md`

---

### Issue 5: Render debug log lines in a dim or gray style

**Problem**  
In `src/ui/logs.rs`, log lines are styled based on keywords like `error`, `warn`, and `info`. Lines containing `debug` are currently rendered the same as normal output, which makes scanning logs harder.

**What to do**
- Update `src/ui/logs.rs` to detect the substring `debug`, case insensitive.
- Render debug lines using a dim or gray style, such as `Color::DarkGray`, consistent with the rest of the UI.
- Keep changes limited to this file.

**How to verify**
- Run the app and view logs that include debug lines.
- Confirm that debug messages appear visually dimmer than info or warning messages.

**Relevant files**
- `src/ui/logs.rs`- [ ] `cargo check` passes (no Docker runtime required).
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
