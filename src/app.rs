//! Main application state and logic
//!
//! Manages the application state, handles messages, and renders views.

use iced::widget::{column, container, row, text};
use iced::{Element, Subscription, Task, Theme};

use crate::cache::{CacheKeys, CacheStore, CacheTTL};
use crate::github::{GitHubClient, MarketplaceClient};
use crate::models::{MarketplaceAction, Workflow};
use crate::ui::{
    action_browser, auth_view, editor_view, empty_state, status_bar, step_editor,
    workflow_sidebar,
};

/// Application state
pub struct App {
    /// Current screen
    screen: Screen,

    /// GitHub API client
    github_client: GitHubClient,

    /// Marketplace client
    marketplace_client: MarketplaceClient,

    /// Cache store
    cache: CacheStore,

    /// Current workflow being edited
    workflow: Option<Workflow>,

    /// Current workflow file path
    workflow_path: Option<String>,

    /// Available marketplace actions (cached)
    marketplace_actions: Vec<MarketplaceAction>,

    /// Action search query
    action_search_query: String,

    /// Selected job ID
    selected_job: Option<String>,

    /// Selected step index within job
    selected_step: Option<usize>,

    /// Token input for auth screen
    token_input: String,

    /// Current error message
    error_message: Option<String>,

    /// Loading state
    is_loading: bool,

    /// Authenticated username
    username: Option<String>,

    /// API rate limit remaining
    rate_limit: Option<u32>,
}

/// Application screens
#[derive(Debug, Clone, PartialEq)]
enum Screen {
    Auth,
    Editor,
    Settings,
}

/// Application messages
#[derive(Debug, Clone)]
pub enum Message {
    // Navigation
    NavigateTo(Screen),

    // Authentication
    TokenInputChanged(String),
    SubmitToken,
    AuthResult(Result<String, String>),
    Logout,

    // Workflow operations
    NewWorkflow,
    OpenWorkflow,
    SaveWorkflow,
    WorkflowLoaded(Result<Workflow, String>),
    WorkflowSaved(Result<(), String>),

    // Job operations
    SelectJob(String),
    AddJob,
    RemoveJob(String),
    JobNameChanged(String, String),
    JobRunsOnChanged(String, String),

    // Step operations
    SelectStep(String, usize),
    AddStep(String),
    RemoveStep(String, usize),
    StepNameChanged(String, usize, String),
    StepUsesChanged(String, usize, String),
    StepRunChanged(String, usize, String),

    // Action browser
    SearchActionsChanged(String),
    ActionSelected(String),
    ActionVersionSelected(String, String),
    MarketplaceLoaded(Result<Vec<MarketplaceAction>, String>),

    // Settings
    ClearCache,
    CacheCleared(Result<(), String>),

    // Misc
    None,
    Error(String),
}

impl App {
    /// Create a new application instance
    pub fn new() -> (Self, Task<Message>) {
        let github_client = GitHubClient::default();
        let marketplace_client = MarketplaceClient::default();
        let cache = CacheStore::default();

        let is_authenticated = github_client.is_authenticated();

        let app = Self {
            screen: if is_authenticated {
                Screen::Editor
            } else {
                Screen::Auth
            },
            github_client,
            marketplace_client,
            cache,
            workflow: None,
            workflow_path: None,
            marketplace_actions: vec![],
            action_search_query: String::new(),
            selected_job: None,
            selected_step: None,
            token_input: String::new(),
            error_message: None,
            is_loading: false,
            username: None,
            rate_limit: None,
        };

        let task = if is_authenticated {
            Task::perform(async { load_featured_actions().await }, |result| {
                Message::MarketplaceLoaded(result)
            })
        } else {
            Task::none()
        };

        (app, task)
    }

    /// Handle messages and update state
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::NavigateTo(screen) => {
                self.screen = screen;
                Task::none()
            }

            // Authentication
            Message::TokenInputChanged(token) => {
                self.token_input = token;
                self.error_message = None;
                Task::none()
            }

            Message::SubmitToken => {
                self.is_loading = true;
                self.error_message = None;
                let token = self.token_input.clone();

                Task::perform(
                    async move { validate_and_store_token(token).await },
                    Message::AuthResult,
                )
            }

            Message::AuthResult(result) => {
                self.is_loading = false;
                match result {
                    Ok(username) => {
                        self.username = Some(username);
                        self.screen = Screen::Editor;
                        self.token_input.clear();

                        // Load featured actions
                        Task::perform(async { load_featured_actions().await }, |result| {
                            Message::MarketplaceLoaded(result)
                        })
                    }
                    Err(err) => {
                        self.error_message = Some(err);
                        Task::none()
                    }
                }
            }

            Message::Logout => {
                let _ = self.github_client.logout();
                self.username = None;
                self.screen = Screen::Auth;
                self.workflow = None;
                self.marketplace_actions.clear();
                Task::none()
            }

            // Workflow operations
            Message::NewWorkflow => {
                self.workflow = Some(Workflow::default());
                self.workflow_path = None;
                self.selected_job = None;
                self.selected_step = None;
                Task::none()
            }

            Message::OpenWorkflow => {
                // TODO: Implement file dialog
                Task::none()
            }

            Message::SaveWorkflow => {
                // TODO: Implement save
                Task::none()
            }

            Message::WorkflowLoaded(result) => {
                match result {
                    Ok(workflow) => {
                        self.workflow = Some(workflow);
                        self.error_message = None;
                    }
                    Err(err) => {
                        self.error_message = Some(err);
                    }
                }
                Task::none()
            }

            Message::WorkflowSaved(result) => {
                if let Err(err) = result {
                    self.error_message = Some(err);
                }
                Task::none()
            }

            // Job operations
            Message::SelectJob(job_id) => {
                self.selected_job = Some(job_id);
                self.selected_step = None;
                Task::none()
            }

            Message::AddJob => {
                if let Some(ref mut workflow) = self.workflow {
                    let job_id = format!("job_{}", workflow.jobs.len() + 1);
                    workflow.jobs.insert(job_id.clone(), Default::default());
                    self.selected_job = Some(job_id);
                }
                Task::none()
            }

            Message::RemoveJob(job_id) => {
                if let Some(ref mut workflow) = self.workflow {
                    workflow.jobs.remove(&job_id);
                    if self.selected_job.as_ref() == Some(&job_id) {
                        self.selected_job = workflow.jobs.keys().next().cloned();
                    }
                }
                Task::none()
            }

            Message::JobNameChanged(job_id, name) => {
                if let Some(ref mut workflow) = self.workflow {
                    if let Some(job) = workflow.jobs.get_mut(&job_id) {
                        job.name = if name.is_empty() { None } else { Some(name) };
                    }
                }
                Task::none()
            }

            Message::JobRunsOnChanged(job_id, runs_on) => {
                if let Some(ref mut workflow) = self.workflow {
                    if let Some(job) = workflow.jobs.get_mut(&job_id) {
                        job.runs_on = crate::models::RunsOn::Single(runs_on);
                    }
                }
                Task::none()
            }

            // Step operations
            Message::SelectStep(job_id, step_idx) => {
                self.selected_job = Some(job_id);
                self.selected_step = Some(step_idx);
                Task::none()
            }

            Message::AddStep(job_id) => {
                if let Some(ref mut workflow) = self.workflow {
                    if let Some(job) = workflow.jobs.get_mut(&job_id) {
                        job.steps.push(Default::default());
                        self.selected_step = Some(job.steps.len() - 1);
                    }
                }
                Task::none()
            }

            Message::RemoveStep(job_id, step_idx) => {
                if let Some(ref mut workflow) = self.workflow {
                    if let Some(job) = workflow.jobs.get_mut(&job_id) {
                        if step_idx < job.steps.len() {
                            job.steps.remove(step_idx);
                            self.selected_step = None;
                        }
                    }
                }
                Task::none()
            }

            Message::StepNameChanged(job_id, step_idx, name) => {
                if let Some(ref mut workflow) = self.workflow {
                    if let Some(job) = workflow.jobs.get_mut(&job_id) {
                        if let Some(step) = job.steps.get_mut(step_idx) {
                            step.name = if name.is_empty() { None } else { Some(name) };
                        }
                    }
                }
                Task::none()
            }

            Message::StepUsesChanged(job_id, step_idx, uses) => {
                if let Some(ref mut workflow) = self.workflow {
                    if let Some(job) = workflow.jobs.get_mut(&job_id) {
                        if let Some(step) = job.steps.get_mut(step_idx) {
                            step.uses = if uses.is_empty() { None } else { Some(uses) };
                            step.run = None; // Clear run when using action
                        }
                    }
                }
                Task::none()
            }

            Message::StepRunChanged(job_id, step_idx, run) => {
                if let Some(ref mut workflow) = self.workflow {
                    if let Some(job) = workflow.jobs.get_mut(&job_id) {
                        if let Some(step) = job.steps.get_mut(step_idx) {
                            step.run = if run.is_empty() { None } else { Some(run) };
                            step.uses = None; // Clear uses when running command
                        }
                    }
                }
                Task::none()
            }

            // Action browser
            Message::SearchActionsChanged(query) => {
                self.action_search_query = query.clone();

                if query.len() >= 2 {
                    Task::perform(
                        async move { search_marketplace_actions(query).await },
                        Message::MarketplaceLoaded,
                    )
                } else {
                    Task::none()
                }
            }

            Message::ActionSelected(action_ref) => {
                // Insert selected action into current step
                if let (Some(job_id), Some(step_idx)) =
                    (self.selected_job.clone(), self.selected_step)
                {
                    if let Some(ref mut workflow) = self.workflow {
                        if let Some(job) = workflow.jobs.get_mut(&job_id) {
                            if let Some(step) = job.steps.get_mut(step_idx) {
                                step.uses = Some(action_ref);
                                step.run = None;
                            }
                        }
                    }
                }
                Task::none()
            }

            Message::ActionVersionSelected(action, version) => {
                // TODO: Update step with versioned action ref
                Task::none()
            }

            Message::MarketplaceLoaded(result) => {
                match result {
                    Ok(actions) => {
                        self.marketplace_actions = actions;
                    }
                    Err(err) => {
                        tracing::warn!("Failed to load marketplace: {}", err);
                    }
                }
                Task::none()
            }

            // Settings
            Message::ClearCache => {
                let result = self.cache.clear();
                Task::done(Message::CacheCleared(result.map_err(|e| e.to_string())))
            }

            Message::CacheCleared(result) => {
                if let Err(err) = result {
                    self.error_message = Some(format!("Failed to clear cache: {}", err));
                }
                Task::none()
            }

            Message::None => Task::none(),

            Message::Error(err) => {
                self.error_message = Some(err);
                Task::none()
            }
        }
    }

    /// Render the application view
    pub fn view(&self) -> Element<Message> {
        match self.screen {
            Screen::Auth => auth_view(
                &self.token_input,
                self.error_message.as_deref(),
                self.is_loading,
                Message::TokenInputChanged,
                Message::SubmitToken,
            ),

            Screen::Editor => {
                if let Some(ref workflow) = self.workflow {
                    let sidebar = workflow_sidebar(
                        workflow,
                        self.selected_job.as_deref(),
                        self.selected_step,
                        Message::SelectJob,
                        |job_id, step_idx| Message::SelectStep(job_id, step_idx),
                        Message::AddJob,
                    );

                    let content = self.render_editor_content(workflow);

                    let cache_stats = self.cache.stats();
                    let status = status_bar(
                        self.username.as_deref(),
                        self.rate_limit,
                        Some(&format!(
                            "Cache: {} entries, {}",
                            cache_stats.disk_entries,
                            cache_stats.disk_size_human()
                        )),
                    );

                    editor_view(sidebar, content, status)
                } else {
                    empty_state(
                        "No Workflow Open",
                        "Create a new workflow or open an existing one to get started.",
                        "New Workflow",
                        Message::NewWorkflow,
                    )
                }
            }

            Screen::Settings => {
                // TODO: Settings view
                text("Settings").into()
            }
        }
    }

    fn render_editor_content(&self, workflow: &Workflow) -> Element<Message> {
        let action_browser_panel = action_browser(
            &self.action_search_query,
            &self.marketplace_actions,
            Message::SearchActionsChanged,
            Message::ActionSelected,
        );

        // Main content area
        if let Some(ref job_id) = self.selected_job {
            if let Some(job) = workflow.jobs.get(job_id) {
                let job_header = column![
                    text(job.name.as_deref().unwrap_or(job_id)).size(20),
                    text(format!("runs-on: {:?}", job.runs_on)).size(12),
                ]
                .spacing(4);

                let steps: Vec<Element<Message>> = job
                    .steps
                    .iter()
                    .enumerate()
                    .map(|(idx, step)| {
                        let job_id = job_id.clone();
                        step_editor(
                            idx,
                            step,
                            {
                                let job_id = job_id.clone();
                                move |idx, name| Message::StepNameChanged(job_id.clone(), idx, name)
                            },
                            {
                                let job_id = job_id.clone();
                                move |idx, uses| Message::StepUsesChanged(job_id.clone(), idx, uses)
                            },
                            {
                                let job_id = job_id.clone();
                                move |idx, run| Message::StepRunChanged(job_id.clone(), idx, run)
                            },
                            {
                                let job_id = job_id.clone();
                                move |idx| Message::RemoveStep(job_id.clone(), idx)
                            },
                        )
                    })
                    .collect();

                row![
                    column![job_header, column(steps).spacing(10)]
                        .spacing(20)
                        .width(iced::Length::FillPortion(2)),
                    container(action_browser_panel)
                        .width(iced::Length::FillPortion(1))
                        .padding(10),
                ]
                .spacing(20)
                .into()
            } else {
                text("Job not found").into()
            }
        } else {
            row![
                text("Select a job from the sidebar").size(16),
                container(action_browser_panel)
                    .width(iced::Length::FillPortion(1))
                    .padding(10),
            ]
            .into()
        }
    }

    /// Handle subscriptions (e.g., keyboard shortcuts)
    pub fn subscription(&self) -> Subscription<Message> {
        Subscription::none()
    }
}

// Async helper functions

async fn validate_and_store_token(token: String) -> Result<String, String> {
    let mut client = GitHubClient::new().map_err(|e| e.to_string())?;
    let validation = client
        .authenticate(&token)
        .await
        .map_err(|e| e.to_string())?;

    if validation.valid {
        Ok(validation.username.unwrap_or_else(|| "Unknown".to_string()))
    } else {
        Err(validation.error.unwrap_or_else(|| "Invalid token".to_string()))
    }
}

async fn load_featured_actions() -> Result<Vec<MarketplaceAction>, String> {
    let client = MarketplaceClient::default();
    client.get_featured().await.map_err(|e| e.to_string())
}

async fn search_marketplace_actions(query: String) -> Result<Vec<MarketplaceAction>, String> {
    let client = MarketplaceClient::default();
    let result = client.search(&query, 1, 30).await.map_err(|e| e.to_string())?;
    Ok(result.items)
}

