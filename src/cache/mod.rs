//! Caching layer for marketplace data and action metadata
//!
//! Provides persistent caching with TTL for GitHub API responses.

mod store;
mod entries;

pub use store::*;
pub use entries::*;

