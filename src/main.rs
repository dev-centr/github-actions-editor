//! GitHub Actions Editor
//!
//! A desktop GUI application for editing GitHub Actions workflows with:
//! - Visual workflow editing using desktop UI metaphors
//! - Marketplace discovery with fuzzy search
//! - GitHub authentication integration
//! - Caching for action metadata and versions

mod app;
mod cache;
mod github;
mod models;
mod ui;

use app::App;
use iced::{Settings, Size};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

fn main() -> iced::Result {
    // Initialize logging
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    tracing::info!("Starting GitHub Actions Editor");

    iced::application("GitHub Actions Editor", App::update, App::view)
        .subscription(App::subscription)
        .window_size(Size::new(1400.0, 900.0))
        .run_with(App::new)
}

