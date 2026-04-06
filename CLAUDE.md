# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## What is absorb

A terminal RSVP (Rapid Serial Visual Presentation) speed reader built in Rust. It displays text one word at a time, aligned on the Optimal Recognition Point (ORP), using a ratatui TUI. Accepts file input or stdin piping.

## Build & Development Commands

```bash
cargo build              # Build debug
cargo build --release    # Build release
cargo test               # Run all tests
cargo test <test_name>   # Run a single test (e.g. cargo test orp_index_short_words)
cargo fmt --check        # Check formatting (CI enforces this)
cargo fmt                # Auto-format
```

CI runs `cargo test` and `cargo fmt --check` on every PR and push to master.

## Architecture

**Single-binary TUI app** using `ratatui` + `crossterm` with a `tokio` async event loop.

- `main.rs` — CLI parsing (clap), input reading (file or stdin), terminal setup/teardown, panic hook
- `app.rs` — Core application state and event loop. `App::run()` uses `tokio::select!` over a timer tick and crossterm `EventStream`. Handles keyboard/mouse input, WPM ease-in ramp (first 10 words), play/pause/navigation
- `display/mod.rs` — Layout orchestration. Splits the frame into word pane + footer (or split-view with text pane). `ViewState` is the read-only snapshot passed to drawing; `DrawResult` returns hit-test data back to `App`
- `display/word.rs` — ORP calculation and single-word rendering. The ORP formula: `(len - 1) / 4 + 1` for words longer than 1 char
- `display/text_view.rs` — Split-view full-text panel with word highlighting and `WordMap` for mouse click hit-testing
- `display/footer.rs` — Progress bar, controls legend, status line (WPM/position/state)
- `banners.rs` — ASCII art header

**Key data flow:** `App` owns state -> builds `ViewState` -> `display::draw()` renders frame -> returns `DrawResult` (word_map, text_pane rect, scroll position) -> `App` uses these for mouse hit-testing.

## Conventions

- Rust 2024 edition
- No external test framework; all tests use `#[cfg(test)]` inline modules
- Display widgets are tested via `render_to_string()` helper in `display/mod.rs::tests`
- Default branch is `master`
