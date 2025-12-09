//! GitHub API client
//!
//! High-level client for interacting with the GitHub API.

use super::auth::{AuthError, AuthManager, TokenValidation};
use crate::models::{ActionMetadata, RepositoryInfo, TokenInfo};
use chrono::{DateTime, Utc};
use octocrab::Octocrab;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ClientError {
    #[error("Authentication error: {0}")]
    Auth(#[from] AuthError),

    #[error("GitHub API error: {0}")]
    Api(#[from] octocrab::Error),

    #[error("Request error: {0}")]
    Request(#[from] reqwest::Error),

    #[error("YAML parsing error: {0}")]
    Yaml(#[from] serde_yaml::Error),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Rate limited. Reset at {0}")]
    RateLimited(DateTime<Utc>),
}

/// GitHub API client with caching support
pub struct GitHubClient {
    auth: AuthManager,
    client: Option<Octocrab>,
}

impl GitHubClient {
    /// Create a new GitHub client
    pub fn new() -> Result<Self, ClientError> {
        let auth = AuthManager::new()?;
        let client = if auth.has_token() {
            let token = auth.get_token()?;
            Some(
                Octocrab::builder()
                    .personal_token(token)
                    .build()
                    .map_err(ClientError::Api)?,
            )
        } else {
            None
        };

        Ok(Self { auth, client })
    }

    /// Check if authenticated
    pub fn is_authenticated(&self) -> bool {
        self.client.is_some()
    }

    /// Authenticate with a GitHub token
    pub async fn authenticate(&mut self, token: &str) -> Result<TokenValidation, ClientError> {
        // Validate the token first
        let validation = self.validate_token(token).await?;

        if validation.valid {
            // Store the token
            self.auth.store_token(token)?;

            // Create the client
            self.client = Some(
                Octocrab::builder()
                    .personal_token(token.to_string())
                    .build()
                    .map_err(ClientError::Api)?,
            );
        }

        Ok(validation)
    }

    /// Validate a GitHub token
    pub async fn validate_token(&self, token: &str) -> Result<TokenValidation, ClientError> {
        let client = reqwest::Client::new();

        let response = client
            .get("https://api.github.com/user")
            .header("Authorization", format!("Bearer {}", token))
            .header("User-Agent", "github-actions-editor")
            .header("Accept", "application/vnd.github+json")
            .send()
            .await?;

        if !response.status().is_success() {
            return Ok(TokenValidation::invalid(format!(
                "HTTP {}: {}",
                response.status(),
                response.text().await.unwrap_or_default()
            )));
        }

        // Extract scopes from header
        let scopes = response
            .headers()
            .get("x-oauth-scopes")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.split(", ").map(String::from).collect())
            .unwrap_or_default();

        let rate_limit = response
            .headers()
            .get("x-ratelimit-remaining")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.parse().ok())
            .unwrap_or(0);

        let user: serde_json::Value = response.json().await?;
        let username = user["login"].as_str().map(String::from);

        Ok(TokenValidation {
            valid: true,
            username,
            scopes,
            rate_limit_remaining: rate_limit,
            error: None,
        })
    }

    /// Log out and delete stored token
    pub fn logout(&mut self) -> Result<(), ClientError> {
        self.auth.delete_token()?;
        self.client = None;
        Ok(())
    }

    /// Get token info
    pub async fn get_token_info(&self) -> Result<TokenInfo, ClientError> {
        let client = self.get_client()?;
        let user = client.current().user().await?;

        // Get rate limit info
        let rate_limit = client.ratelimit().get().await?;

        Ok(TokenInfo {
            scopes: vec![], // Would need separate API call
            expires_at: None,
            user: user.login,
            rate_limit_remaining: rate_limit.rate.remaining as u32,
            rate_limit_reset: DateTime::from_timestamp(rate_limit.rate.reset as i64, 0)
                .unwrap_or_else(Utc::now),
        })
    }

    /// Get repository info
    pub async fn get_repository(&self, owner: &str, repo: &str) -> Result<RepositoryInfo, ClientError> {
        let client = self.get_client()?;
        let repository = client.repos(owner, repo).get().await?;

        Ok(RepositoryInfo {
            full_name: repository.full_name.unwrap_or_default(),
            owner: owner.to_string(),
            name: repo.to_string(),
            private: repository.private.unwrap_or(false),
            default_branch: repository.default_branch.unwrap_or_else(|| "main".to_string()),
            description: repository.description,
            topics: repository.topics.unwrap_or_default(),
        })
    }

    /// Get action metadata from a repository
    pub async fn get_action_metadata(
        &self,
        owner: &str,
        repo: &str,
        ref_: Option<&str>,
    ) -> Result<ActionMetadata, ClientError> {
        let client = self.get_client()?;
        let reference = ref_.unwrap_or("HEAD");

        // Try action.yml first, then action.yaml
        let content = match client
            .repos(owner, repo)
            .get_content()
            .path("action.yml")
            .r#ref(reference)
            .send()
            .await
        {
            Ok(content) => content,
            Err(_) => client
                .repos(owner, repo)
                .get_content()
                .path("action.yaml")
                .r#ref(reference)
                .send()
                .await
                .map_err(|_| ClientError::NotFound("action.yml or action.yaml".to_string()))?,
        };

        // Extract file content
        let file = content
            .items
            .into_iter()
            .next()
            .ok_or_else(|| ClientError::NotFound("action.yml".to_string()))?;

        let decoded = file
            .decoded_content()
            .ok_or_else(|| ClientError::NotFound("Could not decode action.yml".to_string()))?;

        let metadata = ActionMetadata::from_yaml(&decoded)?;
        Ok(metadata)
    }

    /// List user's repositories
    pub async fn list_repositories(&self) -> Result<Vec<RepositoryInfo>, ClientError> {
        let client = self.get_client()?;

        let repos = client
            .current()
            .list_repos_for_authenticated_user()
            .per_page(100)
            .send()
            .await?;

        Ok(repos
            .items
            .into_iter()
            .map(|r| RepositoryInfo {
                full_name: r.full_name.unwrap_or_default(),
                owner: r.owner.map(|o| o.login).unwrap_or_default(),
                name: r.name,
                private: r.private.unwrap_or(false),
                default_branch: r.default_branch.unwrap_or_else(|| "main".to_string()),
                description: r.description,
                topics: r.topics.unwrap_or_default(),
            })
            .collect())
    }

    /// List repository secrets (names only)
    pub async fn list_repository_secrets(
        &self,
        owner: &str,
        repo: &str,
    ) -> Result<Vec<String>, ClientError> {
        let client = self.get_client()?;
        let secrets = client.actions().list_repository_secrets(owner, repo).await?;

        Ok(secrets.secrets.into_iter().map(|s| s.name).collect())
    }

    /// List repository variables (names only)
    pub async fn list_repository_variables(
        &self,
        owner: &str,
        repo: &str,
    ) -> Result<Vec<String>, ClientError> {
        let client = self.get_client()?;
        let vars = client.actions().list_repository_variables(owner, repo).await?;

        Ok(vars.variables.into_iter().map(|v| v.name).collect())
    }

    fn get_client(&self) -> Result<&Octocrab, ClientError> {
        self.client
            .as_ref()
            .ok_or_else(|| ClientError::Auth(super::auth::AuthError::NoToken))
    }
}

impl Default for GitHubClient {
    fn default() -> Self {
        Self::new().expect("Failed to create GitHub client")
    }
}

