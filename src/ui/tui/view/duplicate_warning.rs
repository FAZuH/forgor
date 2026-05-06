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

pub struct DuplicateWarning {}

impl DuplicateWarning {
    pub fn new() -> Self {
        Self {}
    }
}

impl Widget for DuplicateWarning {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let popup_width = 60.min(area.width.saturating_sub(4));
        let popup_height = 10.min(area.height);

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
        ];

        Paragraph::new(text)
            .block(block)
            .alignment(ratatui::layout::Alignment::Center)
            .render(popup_area, buf);
    }
}
