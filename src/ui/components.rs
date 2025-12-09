//! Reusable UI components
//!
//! Common widgets and building blocks for the editor UI.

use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use iced::widget::{
    button, column, container, horizontal_space, pick_list, row, scrollable, text,
    text_input, vertical_space, Column, Row,
};
use iced::{Alignment, Element, Length};

use crate::models::{ActionInput, MarketplaceAction, Step, Workflow};

/// Fuzzy search helper
pub struct FuzzySearch {
    matcher: SkimMatcherV2,
}

impl FuzzySearch {
    pub fn new() -> Self {
        Self {
            matcher: SkimMatcherV2::default(),
        }
    }

    /// Search and score items
    pub fn search<'a, T, F>(&self, query: &str, items: &'a [T], get_text: F) -> Vec<(&'a T, i64)>
    where
        F: Fn(&T) -> &str,
    {
        if query.is_empty() {
            return items.iter().map(|item| (item, 0)).collect();
        }

        let mut results: Vec<_> = items
            .iter()
            .filter_map(|item| {
                self.matcher
                    .fuzzy_match(get_text(item), query)
                    .map(|score| (item, score))
            })
            .collect();

        results.sort_by(|a, b| b.1.cmp(&a.1));
        results
    }

    /// Search marketplace actions
    pub fn search_actions<'a>(
        &self,
        query: &str,
        actions: &'a [MarketplaceAction],
    ) -> Vec<(&'a MarketplaceAction, i64)> {
        self.search(query, actions, |a| &a.full_name)
    }
}

impl Default for FuzzySearch {
    fn default() -> Self {
        Self::new()
    }
}

/// Messages from UI components
#[derive(Debug, Clone)]
pub enum ComponentMessage {
    // Action browser
    SearchQueryChanged(String),
    ActionSelected(String),
    VersionSelected(String, String),
    
    // Step editor
    StepNameChanged(usize, String),
    StepUsesChanged(usize, String),
    StepRunChanged(usize, String),
    StepConditionChanged(usize, String),
    StepInputChanged(usize, String, String),
    AddStep,
    RemoveStep(usize),
    MoveStepUp(usize),
    MoveStepDown(usize),
    
    // Job editor
    JobNameChanged(String, String),
    JobRunsOnChanged(String, String),
    JobNeedsChanged(String, Vec<String>),
    AddJob,
    RemoveJob(String),
    
    // Workflow settings
    WorkflowNameChanged(String),
    TriggerChanged(String),
    
    // Auth
    TokenSubmitted(String),
    Logout,
}

/// Action browser widget for searching and selecting marketplace actions
pub fn action_browser<'a, Message>(
    search_query: &str,
    actions: &[MarketplaceAction],
    on_search: impl Fn(String) -> Message + 'a,
    on_select: impl Fn(String) -> Message + 'a + Clone,
) -> Element<'a, Message>
where
    Message: 'a + Clone,
{
    let search_input = text_input("Search actions...", search_query)
        .on_input(on_search)
        .padding(10)
        .width(Length::Fill);

    let fuzzy = FuzzySearch::new();
    let filtered = fuzzy.search_actions(search_query, actions);

    let action_list: Element<Message> = if filtered.is_empty() {
        container(text("No actions found").size(14))
            .padding(20)
            .center_x(Length::Fill)
            .into()
    } else {
        let items: Vec<Element<Message>> = filtered
            .into_iter()
            .take(50) // Limit results for performance
            .map(|(action, _score)| {
                let on_select = on_select.clone();
                let full_name = action.full_name.clone();
                
                button(
                    column![
                        text(&action.full_name).size(14),
                        text(&action.description)
                            .size(12)
                            .style(iced::widget::text::Style {
                                color: Some(iced::Color::from_rgb(0.5, 0.5, 0.5)),
                            }),
                        row![
                            text(format!("⭐ {}", action.stars)).size(11),
                            horizontal_space(),
                            text(action.categories.join(", ")).size(11),
                        ]
                        .spacing(10)
                    ]
                    .spacing(4)
                    .padding(8),
                )
                .on_press(on_select(full_name))
                .width(Length::Fill)
                .into()
            })
            .collect();

        scrollable(Column::with_children(items).spacing(4))
            .height(Length::Fill)
            .into()
    };

    column![search_input, vertical_space().height(10), action_list]
        .spacing(5)
        .into()
}

/// Step editor widget for editing a workflow step
pub fn step_editor<'a, Message>(
    index: usize,
    step: &Step,
    on_name_change: impl Fn(usize, String) -> Message + 'a,
    on_uses_change: impl Fn(usize, String) -> Message + 'a,
    on_run_change: impl Fn(usize, String) -> Message + 'a,
    on_remove: impl Fn(usize) -> Message + 'a,
) -> Element<'a, Message>
where
    Message: 'a + Clone,
{
    let name_input = text_input(
        "Step name",
        step.name.as_deref().unwrap_or(""),
    )
    .on_input(move |s| on_name_change(index, s))
    .padding(8)
    .width(Length::Fill);

    let step_type = if step.uses.is_some() {
        // Action step
        let uses_input = text_input(
            "Action (e.g., actions/checkout@v4)",
            step.uses.as_deref().unwrap_or(""),
        )
        .on_input(move |s| on_uses_change(index, s))
        .padding(8)
        .width(Length::Fill);

        column![
            text("Uses action:").size(12),
            uses_input,
        ]
        .spacing(4)
    } else {
        // Run step
        let run_input = text_input(
            "Shell command",
            step.run.as_deref().unwrap_or(""),
        )
        .on_input(move |s| on_run_change(index, s))
        .padding(8)
        .width(Length::Fill);

        column![
            text("Run command:").size(12),
            run_input,
        ]
        .spacing(4)
    };

    let remove_btn = button(text("✕").size(14))
        .on_press(on_remove(index))
        .padding(4);

    container(
        column![
            row![
                text(format!("Step {}", index + 1)).size(14),
                horizontal_space(),
                remove_btn,
            ]
            .align_y(Alignment::Center),
            name_input,
            step_type,
        ]
        .spacing(8)
        .padding(12),
    )
    .into()
}

/// Input field for action parameters
pub fn action_input_field<'a, Message>(
    name: &str,
    input_def: &ActionInput,
    current_value: &str,
    on_change: impl Fn(String, String) -> Message + 'a,
) -> Element<'a, Message>
where
    Message: 'a + Clone,
{
    let name_owned = name.to_string();
    let label = row![
        text(name).size(13),
        if input_def.required {
            text(" *").size(13).style(iced::widget::text::Style {
                color: Some(iced::Color::from_rgb(0.8, 0.2, 0.2)),
            })
        } else {
            text("").size(13)
        },
    ];

    let input = text_input(
        input_def.default.as_deref().unwrap_or(""),
        current_value,
    )
    .on_input(move |v| on_change(name_owned.clone(), v))
    .padding(8)
    .width(Length::Fill);

    let description = text(&input_def.description)
        .size(11)
        .style(iced::widget::text::Style {
            color: Some(iced::Color::from_rgb(0.5, 0.5, 0.5)),
        });

    column![label, input, description]
        .spacing(4)
        .into()
}

/// Status indicator badge
pub fn status_badge<'a, Message>(status: &str, color: iced::Color) -> Element<'a, Message>
where
    Message: 'a,
{
    container(
        text(status)
            .size(11)
            .style(iced::widget::text::Style {
                color: Some(iced::Color::WHITE),
            }),
    )
    .padding([2, 8])
    .into()
}

