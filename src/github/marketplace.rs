//! GitHub Actions Marketplace discovery
//!
//! Search and browse actions from the GitHub Marketplace.

use crate::models::{ActionVersion, MarketplaceAction, MarketplaceSearchResult};
use chrono::Utc;
use reqwest::Client;
use serde::Deserialize;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum MarketplaceError {
    #[error("Request error: {0}")]
    Request(#[from] reqwest::Error),

    #[error("API error: {0}")]
    Api(String),

    #[error("Parse error: {0}")]
    Parse(#[from] serde_json::Error),
}

/// GitHub Actions Marketplace client
pub struct MarketplaceClient {
    client: Client,
    token: Option<String>,
}

impl MarketplaceClient {
    /// Create a new marketplace client
    pub fn new(token: Option<String>) -> Self {
        Self {
            client: Client::new(),
            token,
        }
    }

    /// Search for actions in the marketplace
    pub async fn search(
        &self,
        query: &str,
        page: u32,
        per_page: u32,
    ) -> Result<MarketplaceSearchResult, MarketplaceError> {
        // GitHub doesn't have a dedicated marketplace API, so we search repositories
        // with topic:github-actions or topic:action
        let search_query = format!(
            "{} topic:github-actions OR topic:action in:name,description",
            query
        );

        let mut request = self
            .client
            .get("https://api.github.com/search/repositories")
            .query(&[
                ("q", search_query.as_str()),
                ("sort", "stars"),
                ("order", "desc"),
                ("page", &page.to_string()),
                ("per_page", &per_page.to_string()),
            ])
            .header("User-Agent", "github-actions-editor")
            .header("Accept", "application/vnd.github+json");

        if let Some(ref token) = self.token {
            request = request.header("Authorization", format!("Bearer {}", token));
        }

        let response = request.send().await?;

        if !response.status().is_success() {
            return Err(MarketplaceError::Api(format!(
                "HTTP {}: {}",
                response.status(),
                response.text().await.unwrap_or_default()
            )));
        }

        let search_result: GitHubSearchResult = response.json().await?;

        let items: Vec<MarketplaceAction> = search_result
            .items
            .into_iter()
            .map(|repo| MarketplaceAction {
                full_name: repo.full_name.clone(),
                owner: repo.owner.login,
                repo: repo.name,
                name: repo.full_name,
                description: repo.description.unwrap_or_default(),
                versions: vec![], // Will be populated lazily
                stars: repo.stargazers_count,
                updated_at: repo.updated_at.unwrap_or_else(Utc::now),
                verified: false, // Would need additional API call
                categories: repo.topics.unwrap_or_default(),
                metadata: None,
            })
            .collect();

        Ok(MarketplaceSearchResult {
            total_count: search_result.total_count,
            items,
            has_more: search_result.total_count > (page * per_page),
            next_cursor: None,
        })
    }

    /// Get versions/releases for an action
    pub async fn get_versions(
        &self,
        owner: &str,
        repo: &str,
    ) -> Result<Vec<ActionVersion>, MarketplaceError> {
        // Get tags (for version references like v4, v4.1.0)
        let mut request = self
            .client
            .get(&format!(
                "https://api.github.com/repos/{}/{}/tags",
                owner, repo
            ))
            .query(&[("per_page", "50")])
            .header("User-Agent", "github-actions-editor")
            .header("Accept", "application/vnd.github+json");

        if let Some(ref token) = self.token {
            request = request.header("Authorization", format!("Bearer {}", token));
        }

        let response = request.send().await?;

        if !response.status().is_success() {
            return Err(MarketplaceError::Api(format!(
                "HTTP {}: {}",
                response.status(),
                response.text().await.unwrap_or_default()
            )));
        }

        let tags: Vec<GitHubTag> = response.json().await?;

        let mut versions: Vec<ActionVersion> = tags
            .into_iter()
            .map(|tag| {
                let is_major = is_major_version(&tag.name);
                ActionVersion {
                    tag: tag.name,
                    is_latest: false,
                    is_major,
                    released_at: None, // Would need releases API
                    commit_sha: Some(tag.commit.sha),
                }
            })
            .collect();

        // Mark the first major version as latest
        if let Some(major) = versions.iter_mut().find(|v| v.is_major) {
            major.is_latest = true;
        }

        Ok(versions)
    }

    /// Get popular/featured actions
    pub async fn get_featured(&self) -> Result<Vec<MarketplaceAction>, MarketplaceError> {
        // Search for well-known, popular actions
        let popular_actions = [
            "actions/checkout",
            "actions/setup-node",
            "actions/setup-python",
            "actions/cache",
            "actions/upload-artifact",
            "actions/download-artifact",
            "actions/github-script",
            "peaceiris/actions-gh-pages",
            "JamesIves/github-pages-deploy-action",
            "docker/build-push-action",
        ];

        let mut results = Vec::new();

        for action_ref in popular_actions {
            let parts: Vec<&str> = action_ref.split('/').collect();
            if parts.len() == 2 {
                if let Ok(action) = self.get_action_info(parts[0], parts[1]).await {
                    results.push(action);
                }
            }
        }

        Ok(results)
    }

    /// Get detailed info for a specific action
    pub async fn get_action_info(
        &self,
        owner: &str,
        repo: &str,
    ) -> Result<MarketplaceAction, MarketplaceError> {
        let mut request = self
            .client
            .get(&format!(
                "https://api.github.com/repos/{}/{}",
                owner, repo
            ))
            .header("User-Agent", "github-actions-editor")
            .header("Accept", "application/vnd.github+json");

        if let Some(ref token) = self.token {
            request = request.header("Authorization", format!("Bearer {}", token));
        }

        let response = request.send().await?;

        if !response.status().is_success() {
            return Err(MarketplaceError::Api(format!(
                "HTTP {}: {}",
                response.status(),
                response.text().await.unwrap_or_default()
            )));
        }

        let repo_info: GitHubRepo = response.json().await?;
        let versions = self.get_versions(owner, repo).await.unwrap_or_default();

        Ok(MarketplaceAction {
            full_name: repo_info.full_name.clone(),
            owner: repo_info.owner.login,
            repo: repo_info.name,
            name: repo_info.full_name,
            description: repo_info.description.unwrap_or_default(),
            versions,
            stars: repo_info.stargazers_count,
            updated_at: repo_info.updated_at.unwrap_or_else(Utc::now),
            verified: false,
            categories: repo_info.topics.unwrap_or_default(),
            metadata: None,
        })
    }
}

// GitHub API response types

#[derive(Debug, Deserialize)]
struct GitHubSearchResult {
    total_count: u32,
    items: Vec<GitHubRepo>,
}

#[derive(Debug, Deserialize)]
struct GitHubRepo {
    full_name: String,
    name: String,
    owner: GitHubOwner,
    description: Option<String>,
    stargazers_count: u32,
    updated_at: Option<chrono::DateTime<Utc>>,
    topics: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
struct GitHubOwner {
    login: String,
}

#[derive(Debug, Deserialize)]
struct GitHubTag {
    name: String,
    commit: GitHubCommitRef,
}

#[derive(Debug, Deserialize)]
struct GitHubCommitRef {
    sha: String,
}

/// Check if a version tag is a major version (e.g., "v4" vs "v4.1.0")
fn is_major_version(tag: &str) -> bool {
    let tag = tag.trim_start_matches('v');
    !tag.contains('.')
}

impl Default for MarketplaceClient {
    fn default() -> Self {
        Self::new(None)
    }
}

