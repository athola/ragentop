//! Core types, traits, and pure business logic for ragentop.
//!
//! This crate is the **Functional Core** - all functions should be pure
//! (no I/O, no side effects). Port definitions (traits) live here.

pub mod adapter;
pub mod alert;
pub mod burnrate;
pub mod config;
pub mod dag;
pub mod error;
pub mod event;
pub mod multiplexer;
pub mod normalize;
pub mod pricing;
pub mod protocol;
pub mod stats;
pub mod types;

pub use adapter::{Adapter, Capabilities};
pub use config::{AlertConfig, Config, Daemon, DisplayConfig, Tui, Web};
pub use error::{Error, Result};
pub use multiplexer::{Multiplexer, PaneInfo};
pub use protocol::{Request, Response};
pub use types::*;
