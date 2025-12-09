//! GitHub authentication handling
//!
//! Manages GitHub tokens using the system keyring for secure storage.

use keyring::Entry;
use thiserror::Error;

const SERVICE_NAME: &str = "github-actions-editor";
const GITHUB_TOKEN_KEY: &str = "github-token";

#[derive(Error, Debug)]
pub enum AuthError {
    #[error("No token found. Please authenticate first.")]
    NoToken,

    #[error("Keyring error: {0}")]
    Keyring(#[from] keyring::Error),

    #[error("Invalid token: {0}")]
    InvalidToken(String),

    #[error("Token expired")]
    TokenExpired,

    #[error("Insufficient permissions: {0}")]
    InsufficientPermissions(String),
}

/// GitHub authentication manager
pub struct AuthManager {
    entry: Entry,
}

impl AuthManager {
    /// Create a new auth manager
    pub fn new() -> Result<Self, AuthError> {
        let entry = Entry::new(SERVICE_NAME, GITHUB_TOKEN_KEY)?;
        Ok(Self { entry })
    }

    /// Store a GitHub token securely
    pub fn store_token(&self, token: &str) -> Result<(), AuthError> {
        self.entry.set_password(token)?;
        tracing::info!("GitHub token stored securely");
        Ok(())
    }

    /// Retrieve the stored GitHub token
    pub fn get_token(&self) -> Result<String, AuthError> {
        match self.entry.get_password() {
            Ok(token) => Ok(token),
            Err(keyring::Error::NoEntry) => Err(AuthError::NoToken),
            Err(e) => Err(AuthError::Keyring(e)),
        }
    }

    /// Check if a token is stored
    pub fn has_token(&self) -> bool {
        self.entry.get_password().is_ok()
    }

    /// Delete the stored token
    pub fn delete_token(&self) -> Result<(), AuthError> {
        match self.entry.delete_credential() {
            Ok(()) => {
                tracing::info!("GitHub token deleted");
                Ok(())
            }
            Err(keyring::Error::NoEntry) => Ok(()), // Already deleted
            Err(e) => Err(AuthError::Keyring(e)),
        }
    }
}

impl Default for AuthManager {
    fn default() -> Self {
        Self::new().expect("Failed to initialize auth manager")
    }
}

/// Token validation result
#[derive(Debug, Clone)]
pub struct TokenValidation {
    pub valid: bool,
    pub username: Option<String>,
    pub scopes: Vec<String>,
    pub rate_limit_remaining: u32,
    pub error: Option<String>,
}

impl TokenValidation {
    pub fn invalid(error: impl Into<String>) -> Self {
        Self {
            valid: false,
            username: None,
            scopes: vec![],
            rate_limit_remaining: 0,
            error: Some(error.into()),
        }
    }
}

