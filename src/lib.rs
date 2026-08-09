//! A terminal file viewer: browse a tree, preview files with syntax
//! highlighting, scroll PDFs, and read a repository's history.
//!
//! The whole program lives in this library so that benchmarks and integration
//! tests can call into it; `src/main.rs` is only an entry point that parses
//! arguments and drives [`app::App`].
//!
//! Everything is public so benchmarks can reach it, but none of it is a
//! published API, so the lints that police one are off: they would otherwise
//! demand `#[must_use]` and `# Errors` sections on ~80 internal functions.
#![allow(
    clippy::must_use_candidate,
    clippy::missing_errors_doc,
    clippy::missing_panics_doc
)]

pub mod app;
pub mod clipboard;
pub mod config;
pub mod constants;
pub mod event;
pub mod files;
pub mod git;
pub mod highlight;
pub mod theme;
pub mod tui;
pub mod ui;
pub mod util;
pub mod worker;
