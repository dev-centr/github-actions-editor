//! Main application views
//!
//! High-level view components for different screens.

use iced::widget::{
    button, column, container, horizontal_space, row, scrollable, text, text_input,
    vertical_space, Column,
};
use iced::{Alignment, Element, Length};

use super::theme::{button_style, container_style, Palette};
use crate::models::Workflow;

/// Authentication view for GitHub login
pub fn auth_view<'a, Message>(
    token_input: &str,
    error_message: Option<&str>,
    is_loading: bool,
    on_token_change: impl Fn(String) -> Message + 'a,
    on_submit: Message,
) -> Element<'a, Message>
where
    Message: 'a + Clone,
{
    let title = text("GitHub Actions Editor")
        .size(32);

    let subtitle = text("Connect your GitHub account to get started")
        .size(16)
        .style(iced::widget::text::Style {
            color: Some(iced::Color::from_rgb(0.5, 0.5, 0.5)),
        });

    let token_label = text("Personal Access Token").size(14);
    
    let token_field = text_input("ghp_xxxxxxxxxxxx", token_input)
        .on_input(on_token_change)
        .on_submit(on_submit.clone())
        .padding(12)
        .width(Length::Fixed(400.0));

    let token_hint = text("Create a token at GitHub Settings → Developer settings → Personal access tokens")
        .size(12)
        .style(iced::widget::text::Style {
            color: Some(iced::Color::from_rgb(0.5, 0.5, 0.5)),
        });

    let required_scopes = text("Required scopes: repo, read:org")
        .size(12)
        .style(iced::widget::text::Style {
            color: Some(iced::Color::from_rgb(0.5, 0.5, 0.5)),
        });

    let submit_btn = button(
        text(if is_loading { "Connecting..." } else { "Connect" })
            .size(14)
    )
    .on_press_maybe(if is_loading { None } else { Some(on_submit) })
    .padding([10, 24])
    .style(button_style::primary);

    let error_text = if let Some(err) = error_message {
        text(err)
            .size(14)
            .style(iced::widget::text::Style {
                color: Some(iced::Color::from_rgb(0.8, 0.2, 0.2)),
            })
    } else {
        text("")
    };

    let content = column![
        title,
        subtitle,
        vertical_space().height(40),
        token_label,
        token_field,
        token_hint,
        required_scopes,
        vertical_space().height(20),
        submit_btn,
        error_text,
    ]
    .spacing(10)
    .align_x(Alignment::Center)
    .width(Length::Shrink);

    container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .into()
}

/// Main editor view with sidebar and content area
pub fn editor_view<'a, Message>(
    sidebar: Element<'a, Message>,
    content: Element<'a, Message>,
    status_bar: Element<'a, Message>,
) -> Element<'a, Message>
where
    Message: 'a,
{
    let main_row = row![
        container(sidebar)
            .width(Length::Fixed(280.0))
            .height(Length::Fill)
            .style(container_style::sidebar),
        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(20),
    ];

    column![
        main_row,
        container(status_bar)
            .width(Length::Fill)
            .height(Length::Fixed(28.0))
            .padding([4, 12]),
    ]
    .into()
}

/// Sidebar with workflow structure navigation
pub fn workflow_sidebar<'a, Message>(
    workflow: &Workflow,
    selected_job: Option<&str>,
    selected_step: Option<usize>,
    on_job_select: impl Fn(String) -> Message + 'a + Clone,
    on_step_select: impl Fn(String, usize) -> Message + 'a + Clone,
    on_add_job: Message,
) -> Element<'a, Message>
where
    Message: 'a + Clone,
{
    let header = row![
        text("Workflow").size(16),
        horizontal_space(),
        button(text("+").size(14))
            .on_press(on_add_job)
            .padding([4, 8]),
    ]
    .align_y(Alignment::Center)
    .padding([12, 16]);

    let workflow_name = text(workflow.name.as_deref().unwrap_or("Untitled"))
        .size(14)
        .style(iced::widget::text::Style {
            color: Some(iced::Color::from_rgb(0.7, 0.7, 0.7)),
        });

    let jobs: Vec<Element<Message>> = workflow
        .jobs
        .iter()
        .map(|(job_id, job)| {
            let is_selected = selected_job == Some(job_id.as_str());
            let on_job_select = on_job_select.clone();
            let on_step_select = on_step_select.clone();
            let job_id_owned = job_id.clone();

            let job_header = button(
                row![
                    text(job.name.as_deref().unwrap_or(job_id)).size(14),
                    horizontal_space(),
                    text(format!("{} steps", job.steps.len())).size(12),
                ]
                .width(Length::Fill),
            )
            .on_press(on_job_select(job_id.clone()))
            .width(Length::Fill)
            .padding([8, 16]);

            let steps: Vec<Element<Message>> = if is_selected {
                job.steps
                    .iter()
                    .enumerate()
                    .map(|(idx, step)| {
                        let step_name = step
                            .name
                            .as_deref()
                            .or(step.uses.as_deref())
                            .or(step.run.as_deref().map(|s| {
                                if s.len() > 30 {
                                    &s[..30]
                                } else {
                                    s
                                }
                            }))
                            .unwrap_or("Step");

                        let is_step_selected = selected_step == Some(idx);
                        let job_id_for_step = job_id_owned.clone();
                        let on_step_select = on_step_select.clone();

                        button(text(step_name).size(12))
                            .on_press(on_step_select(job_id_for_step, idx))
                            .width(Length::Fill)
                            .padding([6, 24])
                            .into()
                    })
                    .collect()
            } else {
                vec![]
            };

            column![job_header]
                .push(Column::with_children(steps))
                .into()
        })
        .collect();

    let jobs_list = scrollable(
        Column::with_children(jobs)
            .spacing(2)
            .width(Length::Fill),
    )
    .height(Length::Fill);

    column![
        header,
        container(workflow_name).padding([0, 16]),
        vertical_space().height(10),
        jobs_list,
    ]
    .spacing(4)
    .into()
}

/// Empty state view
pub fn empty_state<'a, Message>(
    title: &str,
    description: &str,
    action_label: &str,
    on_action: Message,
) -> Element<'a, Message>
where
    Message: 'a + Clone,
{
    let content = column![
        text(title).size(24),
        text(description)
            .size(14)
            .style(iced::widget::text::Style {
                color: Some(iced::Color::from_rgb(0.5, 0.5, 0.5)),
            }),
        vertical_space().height(20),
        button(text(action_label).size(14))
            .on_press(on_action)
            .padding([10, 24])
            .style(button_style::primary),
    ]
    .spacing(10)
    .align_x(Alignment::Center);

    container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .into()
}

/// Status bar showing connection status and cache info
pub fn status_bar<'a, Message>(
    username: Option<&str>,
    rate_limit: Option<u32>,
    cache_stats: Option<&str>,
) -> Element<'a, Message>
where
    Message: 'a,
{
    let palette = Palette::default();

    let auth_status = if let Some(user) = username {
        row![
            text("●").size(10).style(iced::widget::text::Style {
                color: Some(palette.success),
            }),
            text(format!("Connected as {}", user)).size(12),
        ]
        .spacing(6)
    } else {
        row![
            text("●").size(10).style(iced::widget::text::Style {
                color: Some(palette.warning),
            }),
            text("Not connected").size(12),
        ]
        .spacing(6)
    };

    let rate_limit_text = rate_limit
        .map(|r| format!("API: {}/5000", r))
        .unwrap_or_default();

    let cache_text = cache_stats.unwrap_or("");

    row![
        auth_status,
        horizontal_space(),
        text(cache_text).size(11).style(iced::widget::text::Style {
            color: Some(palette.text_muted),
        }),
        text(rate_limit_text).size(11).style(iced::widget::text::Style {
            color: Some(palette.text_muted),
        }),
    ]
    .spacing(20)
    .align_y(Alignment::Center)
    .into()
}

