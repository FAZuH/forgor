use ratatui::prelude::*;
use ratatui::symbols::border;
use ratatui::widgets::Block;
use ratatui::widgets::Borders;
use ratatui::widgets::Paragraph;
use tui_widgets::prompts::TextState;

use crate::ui::prelude::*;

pub struct SettingsPrompt {
    pub text_state: TextState<'static>,
    pub label: String,
}

/// Represents a section with border
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Section {
    pub title: String,
    pub section: SettingsSection,
    pub sel_item: SettingsItem,
    pub height: u16,
    pub rows: Vec<SectionRow>,
}

impl Section {
    fn border_color(&self) -> Color {
        if self.sel_item.section() == self.section {
            Color::Green
        } else {
            Color::White
        }
    }
}

/// Individual row within a section
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum SectionRow {
    Blank,
    SubSectionHeader(String),
    Input {
        label: String,
        value: String,
        is_selected: bool,
        warning: bool,
    },
    Checkbox {
        label: String,
        value: bool,
        is_selected: bool,
    },
}

impl SectionRow {
    pub(crate) fn height(&self) -> u16 {
        match self {
            Self::Blank => 1,
            _ => 1,
        }
    }
}

impl Widget for Section {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let style = Style::default()
            .fg(Color::Green)
            .add_modifier(Modifier::BOLD);

        // Create block with border
        let block = Block::default()
            .title(self.title.clone())
            .title_style(style)
            .borders(Borders::ALL)
            .border_set(border::ROUNDED)
            .border_style(Style::default().fg(self.border_color()));

        // Get inner area for content
        let inner = block.inner(area);
        let inner = Rect::new(inner.x, inner.y, inner.width, inner.height);

        // Render the block
        block.render(area, buf);

        // Render rows inside the block
        let mut y = inner.y;
        for row in self.rows {
            let row_height = row.height();
            let row_area = Rect::new(inner.x, y, inner.width, row_height);
            row.render(row_area, buf);
            y += row_height;
        }
    }
}

impl Widget for SectionRow {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        match self {
            SectionRow::Blank => Line::from("").render(area, buf),
            SectionRow::SubSectionHeader(label) => {
                let line = Line::from(vec![
                    Span::styled(
                        " ",
                        Style::default()
                            .fg(Color::White)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(
                        format!("▸{label} "),
                        Style::default()
                            .fg(Color::White)
                            .add_modifier(Modifier::BOLD)
                            .add_modifier(Modifier::UNDERLINED),
                    ),
                ]);
                Paragraph::new(line).render(area, buf);
            }
            SectionRow::Input {
                label,
                value,
                is_selected,
                warning,
            } => {
                let bg = if is_selected {
                    Style::default().bg(Color::DarkGray)
                } else {
                    Style::default()
                };

                let fg = if warning {
                    Style::default().fg(Color::LightRed)
                } else {
                    Style::default()
                };

                let line = Line::from(vec![
                    Span::styled(
                        format!(" {label}: "),
                        Style::default().add_modifier(Modifier::DIM).patch(bg),
                    ),
                    Span::styled(value, Style::default().patch(bg).patch(fg)),
                ]);
                Paragraph::new(line).render(area, buf);
            }
            SectionRow::Checkbox {
                label,
                value,
                is_selected,
            } => {
                let bg = if is_selected {
                    Style::default().bg(Color::DarkGray)
                } else {
                    Style::default()
                };

                let style = Style::default().fg(Color::Cyan).patch(bg);
                let checkbox = if value {
                    Span::styled(" [x] ", style)
                } else {
                    Span::styled(" [ ] ", style)
                };

                let line = Line::from(vec![
                    checkbox,
                    Span::styled("", bg),
                    Span::styled(label.clone(), bg).add_modifier(Modifier::DIM),
                ]);
                Paragraph::new(line).render(area, buf);
            }
        }
    }
}
