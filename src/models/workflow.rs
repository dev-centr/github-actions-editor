//! GitHub Actions Workflow data model
//!
//! Represents the structure of a GitHub Actions workflow YAML file.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A complete GitHub Actions workflow
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Workflow {
    /// Workflow name displayed in GitHub UI
    pub name: Option<String>,

    /// Events that trigger the workflow
    pub on: WorkflowTrigger,

    /// Environment variables available to all jobs
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub env: HashMap<String, String>,

    /// Default settings for all jobs
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub defaults: Option<WorkflowDefaults>,

    /// Concurrency settings
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub concurrency: Option<Concurrency>,

    /// Permissions for the GITHUB_TOKEN
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub permissions: Option<Permissions>,

    /// Jobs to run
    pub jobs: HashMap<String, Job>,
}

/// Workflow trigger configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(untagged)]
pub enum WorkflowTrigger {
    /// Single event name
    #[default]
    None,
    Single(String),
    /// List of event names
    List(Vec<String>),
    /// Detailed event configuration
    Detailed(HashMap<String, Option<TriggerConfig>>),
}

/// Configuration for a specific trigger event
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TriggerConfig {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub branches: Vec<String>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub paths: Vec<String>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub types: Vec<String>,

    /// Cron schedule for schedule trigger
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cron: Option<String>,
}

/// Default settings for jobs
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct WorkflowDefaults {
    pub run: Option<RunDefaults>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RunDefaults {
    pub shell: Option<String>,
    #[serde(rename = "working-directory")]
    pub working_directory: Option<String>,
}

/// Concurrency control settings
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Concurrency {
    /// Simple group name
    Group(String),
    /// Detailed concurrency settings
    Detailed {
        group: String,
        #[serde(rename = "cancel-in-progress", default)]
        cancel_in_progress: bool,
    },
}

/// Permissions for the GITHUB_TOKEN
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Permissions {
    /// Read-all or write-all
    Level(String),
    /// Per-scope permissions
    Detailed(HashMap<String, String>),
}

/// A job in the workflow
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Job {
    /// Human-readable job name
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// Runner to execute the job on
    #[serde(rename = "runs-on")]
    pub runs_on: RunsOn,

    /// Jobs that must complete before this one
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub needs: Option<JobNeeds>,

    /// Condition for running the job
    #[serde(rename = "if", default, skip_serializing_if = "Option::is_none")]
    pub condition: Option<String>,

    /// Environment variables for this job
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub env: HashMap<String, String>,

    /// Job-level permissions
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub permissions: Option<Permissions>,

    /// Deployment environment
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub environment: Option<JobEnvironment>,

    /// Job outputs
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub outputs: HashMap<String, String>,

    /// Strategy for matrix builds
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub strategy: Option<Strategy>,

    /// Container to run the job in
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub container: Option<Container>,

    /// Service containers
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub services: HashMap<String, Container>,

    /// Steps to execute
    #[serde(default)]
    pub steps: Vec<Step>,

    /// Timeout in minutes
    #[serde(rename = "timeout-minutes", default, skip_serializing_if = "Option::is_none")]
    pub timeout_minutes: Option<u32>,

    /// Continue on error
    #[serde(rename = "continue-on-error", default)]
    pub continue_on_error: bool,
}

/// Runner specification
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RunsOn {
    #[default]
    None,
    /// Single runner label
    Single(String),
    /// Multiple runner labels
    Labels(Vec<String>),
    /// Group specification
    Group {
        group: String,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        labels: Vec<String>,
    },
}

/// Job dependencies
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum JobNeeds {
    Single(String),
    Multiple(Vec<String>),
}

/// Deployment environment configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum JobEnvironment {
    Name(String),
    Detailed { name: String, url: Option<String> },
}

/// Matrix strategy configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Strategy {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub matrix: Option<serde_yaml::Value>,

    #[serde(rename = "fail-fast", default)]
    pub fail_fast: bool,

    #[serde(rename = "max-parallel", default, skip_serializing_if = "Option::is_none")]
    pub max_parallel: Option<u32>,
}

/// Container configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Container {
    Image(String),
    Detailed {
        image: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        credentials: Option<ContainerCredentials>,
        #[serde(default, skip_serializing_if = "HashMap::is_empty")]
        env: HashMap<String, String>,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        ports: Vec<String>,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        volumes: Vec<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        options: Option<String>,
    },
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ContainerCredentials {
    pub username: String,
    pub password: String,
}

/// A step in a job
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Step {
    /// Step identifier
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,

    /// Human-readable step name
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// Condition for running the step
    #[serde(rename = "if", default, skip_serializing_if = "Option::is_none")]
    pub condition: Option<String>,

    /// Action to use (mutually exclusive with run)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub uses: Option<String>,

    /// Shell command to run (mutually exclusive with uses)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub run: Option<String>,

    /// Shell to use for run commands
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub shell: Option<String>,

    /// Working directory
    #[serde(rename = "working-directory", default, skip_serializing_if = "Option::is_none")]
    pub working_directory: Option<String>,

    /// Inputs for the action
    #[serde(rename = "with", default, skip_serializing_if = "HashMap::is_empty")]
    pub inputs: HashMap<String, serde_yaml::Value>,

    /// Environment variables
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub env: HashMap<String, String>,

    /// Continue on error
    #[serde(rename = "continue-on-error", default)]
    pub continue_on_error: bool,

    /// Timeout in minutes
    #[serde(rename = "timeout-minutes", default, skip_serializing_if = "Option::is_none")]
    pub timeout_minutes: Option<u32>,
}

impl Workflow {
    /// Parse a workflow from YAML string
    pub fn from_yaml(yaml: &str) -> Result<Self, serde_yaml::Error> {
        serde_yaml::from_str(yaml)
    }

    /// Serialize workflow to YAML string
    pub fn to_yaml(&self) -> Result<String, serde_yaml::Error> {
        serde_yaml::to_string(self)
    }
}

