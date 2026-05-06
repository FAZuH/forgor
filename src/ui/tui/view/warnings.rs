use ratatui::prelude::*;
use ratatui::symbols::border;
use tui_widgets::popup::Popup;

/// A modal overlay warning about a duplicate application instance.
pub struct DuplicateWarning {}

impl DuplicateWarning {
    pub fn new() -> Self {
        Self {}
    }
}

impl Widget for DuplicateWarning {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let text = Text::from(vec![
            Line::from(""),
            Line::from(format!(
                "Another instance of {} is currently running.",
                crate::APP_NAME
            )),
            Line::from(""),
            Line::from("Running multiple instances may lead to data corruption"),
            Line::from("or overwritten settings."),
            Line::from(""),
            Line::from(vec![
                Span::styled("Press ", Style::default()),
                Span::styled("[Enter]", Style::default().fg(Color::Cyan)),
                Span::styled(" to continue anyway or ", Style::default()),
                Span::styled("[q]", Style::default().fg(Color::Red)),
                Span::styled(" to quit.", Style::default()),
            ]),
            Line::from(""),
        ])
        .alignment(Alignment::Center);

        Widget::render(
            Popup::new(text)
                .title(" Warning ")
                .border_style(Style::new().fg(Color::Yellow).add_modifier(Modifier::BOLD))
                .border_set(border::ROUNDED),
            area,
            buf,
        );
    }
}

/// A modal overlay warning about unsaved configuration changes.
pub struct UnsavedWarning {}

impl UnsavedWarning {
    pub fn new() -> Self {
        Self {}
    }
}

impl Widget for UnsavedWarning {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let text = Text::from(vec![
            Line::from(""),
            Line::from("You have unsaved changes on your settings."),
            Line::from(""),
            Line::from(vec![
                Span::styled("Press ", Style::default()),
                Span::styled("[Enter/s]", Style::default().fg(Color::Cyan)),
                Span::styled(" to save and quit, ", Style::default()),
                Span::styled("[Esc]", Style::default().fg(Color::Yellow)),
                Span::styled(" to cancel, ", Style::default()),
            ]),
            Line::from(vec![
                Span::styled("or press ", Style::default()),
                Span::styled("[q]", Style::default().fg(Color::Red)),
                Span::styled(" to quit anyway.", Style::default()),
            ]),
            Line::from(""),
        ])
        .alignment(Alignment::Center);

        Widget::render(
            Popup::new(text)
                .title(" Warning ")
                .border_style(Style::new().fg(Color::Yellow).add_modifier(Modifier::BOLD))
                .border_set(border::ROUNDED),
            area,
            buf,
        );
    }
}

/// A modal overlay confirming the user wants to reset the timer.
pub struct ResetWarning {}

impl ResetWarning {
    pub fn new() -> Self {
        Self {}
    }
}

impl Widget for ResetWarning {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let text = Text::from(vec![
            Line::from(""),
            Line::from("Are you sure you want to reset the timer?"),
            Line::from(""),
            Line::from(vec![
                Span::styled("Press ", Style::default()),
                Span::styled("[Enter/y]", Style::default().fg(Color::Cyan)),
                Span::styled(" to proceed ", Style::default()),
                Span::styled("[Esc/n]", Style::default().fg(Color::Red)),
                Span::styled(" to cancel.", Style::default()),
            ]),
            Line::from(""),
        ])
        .alignment(Alignment::Center);

        Widget::render(
            Popup::new(text)
                .title(" Warning ")
                .border_style(Style::new().fg(Color::Yellow).add_modifier(Modifier::BOLD))
                .border_set(border::ROUNDED),
            area,
            buf,
        );
    }
}
