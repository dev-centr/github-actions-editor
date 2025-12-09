//! GitHub Action metadata model
//!
//! Represents the action.yml metadata for a GitHub Action.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Metadata for a GitHub Action (from action.yml)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ActionMetadata {
    /// Action name
    pub name: String,

    /// Action description
    #[serde(default)]
    pub description: String,

    /// Action author
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,

    /// Input parameters
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub inputs: HashMap<String, ActionInput>,

    /// Output values
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub outputs: HashMap<String, ActionOutput>,

    /// How the action runs
    pub runs: ActionRuns,

    /// Branding for the marketplace
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub branding: Option<ActionBranding>,
}

/// An input parameter for an action
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ActionInput {
    /// Input description
    #[serde(default)]
    pub description: String,

    /// Whether the input is required
    #[serde(default)]
    pub required: bool,

    /// Default value
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default: Option<String>,

    /// Deprecation message
    #[serde(rename = "deprecationMessage", default, skip_serializing_if = "Option::is_none")]
    pub deprecation_message: Option<String>,
}

/// An output value from an action
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ActionOutput {
    /// Output description
    #[serde(default)]
    pub description: String,

    /// Value expression (for composite actions)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
}

/// How an action runs
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "using")]
pub enum ActionRuns {
    /// JavaScript action
    #[serde(rename = "node20")]
    Node20 {
        main: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pre: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        post: Option<String>,
    },
    #[serde(rename = "node16")]
    Node16 {
        main: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pre: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        post: Option<String>,
    },
    /// Docker action
    #[serde(rename = "docker")]
    Docker {
        image: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        entrypoint: Option<String>,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        args: Vec<String>,
        #[serde(default, skip_serializing_if = "HashMap::is_empty")]
        env: HashMap<String, String>,
    },
    /// Composite action
    #[serde(rename = "composite")]
    Composite {
        steps: Vec<super::workflow::Step>,
    },
}

impl Default for ActionRuns {
    fn default() -> Self {
        ActionRuns::Composite { steps: vec![] }
    }
}

/// Branding for marketplace display
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ActionBranding {
    /// Icon name (from Feather icons)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,

    /// Background color
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
}

impl ActionMetadata {
    /// Parse action metadata from YAML string
    pub fn from_yaml(yaml: &str) -> Result<Self, serde_yaml::Error> {
        serde_yaml::from_str(yaml)
    }
}

