//! GitHub Actions Marketplace models
//!
//! Data structures for marketplace actions and search results.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// A GitHub Action from the marketplace
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketplaceAction {
    /// Full action reference (owner/repo)
    pub full_name: String,

    /// Repository owner
    pub owner: String,

    /// Repository name
    pub repo: String,

    /// Action name from action.yml
    pub name: String,

    /// Action description
    pub description: String,

    /// Available versions/tags
    pub versions: Vec<ActionVersion>,

    /// Star count
    pub stars: u32,

    /// Last updated timestamp
    pub updated_at: DateTime<Utc>,

    /// Verified creator badge
    pub verified: bool,

    /// Categories/topics
    pub categories: Vec<String>,

    /// Action metadata (if fetched)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<super::ActionMetadata>,
}

/// A version/release of an action
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionVersion {
    /// Version tag (e.g., "v4", "v4.1.0")
    pub tag: String,

    /// Whether this is the latest version
    pub is_latest: bool,

    /// Whether this is a major version tag (e.g., "v4" vs "v4.1.0")
    pub is_major: bool,

    /// Release date
    #[serde(skip_serializing_if = "Option::is_none")]
    pub released_at: Option<DateTime<Utc>>,

    /// Commit SHA this version points to
    #[serde(skip_serializing_if = "Option::is_none")]
    pub commit_sha: Option<String>,
}

/// Search result from marketplace
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketplaceSearchResult {
    /// Total count of matching actions
    pub total_count: u32,

    /// Search results
    pub items: Vec<MarketplaceAction>,

    /// Whether there are more results
    pub has_more: bool,

    /// Next page cursor
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_cursor: Option<String>,
}

/// GitHub repository info (for token/org discovery)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositoryInfo {
    /// Full repository name (owner/repo)
    pub full_name: String,

    /// Repository owner
    pub owner: String,

    /// Repository name
    pub name: String,

    /// Whether it's a private repository
    pub private: bool,

    /// Default branch
    pub default_branch: String,

    /// Repository description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Topics/tags
    pub topics: Vec<String>,
}

/// GitHub organization info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrganizationInfo {
    /// Organization login name
    pub login: String,

    /// Display name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// Description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Available secrets (names only)
    pub secrets: Vec<String>,

    /// Available variables (names only)
    pub variables: Vec<String>,
}

/// Authentication token info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenInfo {
    /// Token scopes
    pub scopes: Vec<String>,

    /// Token expiration (if applicable)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<DateTime<Utc>>,

    /// User associated with token
    pub user: String,

    /// Rate limit remaining
    pub rate_limit_remaining: u32,

    /// Rate limit reset time
    pub rate_limit_reset: DateTime<Utc>,
}

/// Popular/featured actions for quick access
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeaturedActions {
    /// Checkout and setup actions
    pub setup: Vec<MarketplaceAction>,

    /// Build and test actions
    pub build: Vec<MarketplaceAction>,

    /// Deployment actions
    pub deploy: Vec<MarketplaceAction>,

    /// Utility actions
    pub utility: Vec<MarketplaceAction>,
}

impl MarketplaceAction {
    /// Get the action reference for use in workflows (e.g., "actions/checkout@v4")
    pub fn action_ref(&self, version: Option<&str>) -> String {
        let ver = version
            .map(|v| v.to_string())
            .or_else(|| {
                self.versions
                    .iter()
                    .find(|v| v.is_latest && v.is_major)
                    .map(|v| v.tag.clone())
            })
            .or_else(|| self.versions.first().map(|v| v.tag.clone()))
            .unwrap_or_else(|| "main".to_string());

        format!("{}@{}", self.full_name, ver)
    }
}

