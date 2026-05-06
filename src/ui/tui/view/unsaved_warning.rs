use ratatui::layout::Constraint;
use ratatui::layout::Flex;
use ratatui::layout::Layout;
use ratatui::prelude::*;
use ratatui::text::Line;
use ratatui::text::Span;
use ratatui::widgets::Block;
use ratatui::widgets::Borders;
use ratatui::widgets::Clear;
use ratatui::widgets::Paragraph;

pub struct UnsavedWarning {}

impl UnsavedWarning {
    pub fn new() -> Self {
        Self {}
    }
}

impl Widget for UnsavedWarning {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let popup_width = 60.min(area.width.saturating_sub(4));
        let popup_height = 7.min(area.height);

        let vertical = Layout::vertical([Constraint::Length(popup_height)]).flex(Flex::Center);
        let horizontal = Layout::horizontal([Constraint::Length(popup_width)]).flex(Flex::Center);

        let [popup_area] = vertical.areas(area);
        let [popup_area] = horizontal.areas(popup_area);

        Clear.render(popup_area, buf);

        let block = Block::default()
            .title(" Warning ")
            .title_alignment(HorizontalAlignment::Center)
            .borders(Borders::ALL)
            .border_style(
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            );

        let text = vec![
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
        ];

        Paragraph::new(text)
            .block(block)
            .alignment(ratatui::layout::Alignment::Center)
            .render(popup_area, buf);
    }
}
