//! GitHub API integration
//!
//! Provides authentication, marketplace discovery, and repository operations.

mod auth;
mod client;
mod marketplace;

pub use auth::*;
pub use client::*;
pub use marketplace::*;

