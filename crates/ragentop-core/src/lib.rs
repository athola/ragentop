//! Core types, traits, and pure business logic for ragentop.
//!
//! This crate is the **Functional Core** - all functions should be pure
//! (no I/O, no side effects). Port definitions (traits) live here.

pub mod adapter;
pub mod config;
pub mod dag;
pub mod error;
pub mod types;

pub use adapter::{AdapterCapabilities, AgentAdapter};
pub use config::Config;
pub use error::{Error, Result};
pub use types::*;
