//! Route wiring only: each submodule maps URL paths to the handlers already
//! defined in their matching top-level module (`crate::movies`,
//! `crate::series`, ...) and returns a `Router<AppState>` for `main` to
//! merge. No handler logic lives here — moving a route means editing one
//! `.route(...)` line, not hunting through `main.rs`.
pub mod auth;
pub mod downloads;
pub mod indexers;
pub mod library;
pub mod movies;
pub mod profiles;
pub mod public;
pub mod scheduling;
pub mod series;
pub mod settings;
