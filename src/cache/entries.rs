//! Cache entry types and keys
//!
//! Defines cache keys and TTL values for different types of data.

use chrono::Duration;

/// Cache key generators
pub struct CacheKeys;

impl CacheKeys {
    /// Key for marketplace search results
    pub fn marketplace_search(query: &str, page: u32) -> String {
        format!("marketplace_search_{}_{}", query.to_lowercase().replace(' ', "_"), page)
    }

    /// Key for action metadata
    pub fn action_metadata(owner: &str, repo: &str, version: Option<&str>) -> String {
        match version {
            Some(v) => format!("action_metadata_{}_{}_{}", owner, repo, v),
            None => format!("action_metadata_{}_{}_latest", owner, repo),
        }
    }

    /// Key for action versions
    pub fn action_versions(owner: &str, repo: &str) -> String {
        format!("action_versions_{}_{}", owner, repo)
    }

    /// Key for repository info
    pub fn repository_info(owner: &str, repo: &str) -> String {
        format!("repo_info_{}_{}", owner, repo)
    }

    /// Key for repository secrets list
    pub fn repository_secrets(owner: &str, repo: &str) -> String {
        format!("repo_secrets_{}_{}", owner, repo)
    }

    /// Key for repository variables list
    pub fn repository_variables(owner: &str, repo: &str) -> String {
        format!("repo_vars_{}_{}", owner, repo)
    }

    /// Key for user repositories list
    pub fn user_repositories() -> String {
        "user_repositories".to_string()
    }

    /// Key for featured/popular actions
    pub fn featured_actions() -> String {
        "featured_actions".to_string()
    }

    /// Key for organization info
    pub fn organization_info(org: &str) -> String {
        format!("org_info_{}", org)
    }
}

/// Cache TTL values for different types of data
pub struct CacheTTL;

impl CacheTTL {
    /// TTL for marketplace search results (15 minutes)
    pub fn marketplace_search() -> Duration {
        Duration::minutes(15)
    }

    /// TTL for action metadata (1 hour)
    pub fn action_metadata() -> Duration {
        Duration::hours(1)
    }

    /// TTL for action versions (30 minutes)
    pub fn action_versions() -> Duration {
        Duration::minutes(30)
    }

    /// TTL for repository info (30 minutes)
    pub fn repository_info() -> Duration {
        Duration::minutes(30)
    }

    /// TTL for secrets/variables lists (5 minutes - sensitive)
    pub fn secrets_and_variables() -> Duration {
        Duration::minutes(5)
    }

    /// TTL for user repositories (15 minutes)
    pub fn user_repositories() -> Duration {
        Duration::minutes(15)
    }

    /// TTL for featured actions (1 day)
    pub fn featured_actions() -> Duration {
        Duration::days(1)
    }

    /// TTL for organization info (1 hour)
    pub fn organization_info() -> Duration {
        Duration::hours(1)
    }

    /// Grace period for stale-while-revalidate (1 hour)
    pub fn stale_grace_period() -> Duration {
        Duration::hours(1)
    }
}

