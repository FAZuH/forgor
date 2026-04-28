use std::borrow::Cow;

use ratatui::layout::Flex;
use ratatui::prelude::*;
use ratatui::widgets::Block;
use ratatui::widgets::Borders;
use ratatui::widgets::Clear;
use ratatui::widgets::Paragraph;
use tui_widgets::big_text::BigText;
use tui_widgets::big_text::PixelSize;
use tui_widgets::prompts::FocusState;
use tui_widgets::prompts::Prompt;
use tui_widgets::prompts::State as PromptState;
use tui_widgets::prompts::TextPrompt;
use tui_widgets::prompts::TextState;
use tui_widgets::scrollview::ScrollView;
use tui_widgets::scrollview::ScrollViewState;
use tui_widgets::scrollview::ScrollbarVisibility;

use crate::config::Config;
use crate::config::pomodoro::Alarm;
use crate::config::pomodoro::Alarms;
use crate::config::pomodoro::Hooks;
use crate::config::pomodoro::PomodoroConfig;
use crate::config::pomodoro::Timers;
use crate::ui::update::settings::SETTINGS_VIEW_ITEMS;

pub struct TuiSettingsRenderer {
    scroll_state: ScrollViewState,
    selected_idx: u32,
    pub prompt: Option<SettingsPrompt>,
    has_unsaved_changes: bool,
}

impl TuiSettingsRenderer {
    pub fn new() -> Self {
        Self {
            scroll_state: ScrollViewState::default(),
            selected_idx: 0,
            prompt: None,
            has_unsaved_changes: false,
        }
    }

    /// Move selection up
    pub fn select_up(&mut self) {
        self.selected_idx = self
            .selected_idx
            .saturating_sub(1)
            .clamp(0, SETTINGS_VIEW_ITEMS - 1); // 13 items total
    }

    /// Move selection down
    pub fn select_down(&mut self) {
        self.selected_idx = self
            .selected_idx
            .saturating_add(1)
            .clamp(0, SETTINGS_VIEW_ITEMS - 1);
    }

    /// Scroll up by one row
    pub fn scroll_up(&mut self) {
        self.scroll_state.scroll_up();
    }

    /// Scroll down by one row
    pub fn scroll_down(&mut self) {
        self.scroll_state.scroll_down();
    }

    /// Start editing the currently selected field
    pub fn start_editing_for_field(&mut self, config: &PomodoroConfig) {
        let (label, value) = match self.selected_idx {
            0 => ("Focus", format!("{}", config.timer.focus.as_secs() / 60)),
            1 => (
                "Short Break",
                format!("{}", config.timer.short.as_secs() / 60),
            ),
            2 => (
                "Long Break",
                format!("{}", config.timer.long.as_secs() / 60),
            ),
            3 => (
                "Long Break Interval",
                format!("{}", config.timer.long_interval),
            ),
            7 => ("Focus Hook", config.hook.focus.clone()),
            8 => ("Short Break Hook", config.hook.short.clone()),
            9 => ("Long Break Hook", config.hook.long.clone()),
            10 => ("Focus Alarm", get_alarm_path_value(&config.alarm.focus)),
            11 => (
                "Short Break Alarm",
                get_alarm_path_value(&config.alarm.short),
            ),
            12 => ("Long Break Alarm", get_alarm_path_value(&config.alarm.long)),
            13 => (
                "Focus Alarm Volume",
                get_alarm_volume_value(&config.alarm.focus),
            ),
            14 => (
                "Short Break Alarm Volume",
                get_alarm_volume_value(&config.alarm.short),
            ),
            15 => (
                "Long Break Alarm Volume",
                get_alarm_volume_value(&config.alarm.long),
            ),
            _ => return, // Cannot edit toggles or out of bounds
        };

        let value_len = value.len();
        let mut text_state = TextState::new()
            .with_focus(FocusState::Focused)
            .with_value(value);
        *PromptState::position_mut(&mut text_state) = value_len;

        self.prompt = Some(SettingsPrompt {
            text_state,
            label: label.to_string(),
        });
    }

    /// Cancel editing
    pub fn cancel_editing(&mut self) {
        self.prompt = None;
    }

    /// Get current selection index
    pub fn selected_idx(&self) -> u32 {
        self.selected_idx
    }

    /// Check if currently editing
    pub fn is_editing(&self) -> bool {
        self.prompt.is_some()
    }

    pub fn prompt_state_mut(&mut self) -> Option<&mut SettingsPrompt> {
        self.prompt.as_mut()
    }

    /// Take the finalized edit value
    pub fn take_edit_value(&mut self) -> String {
        if let Some(prompt) = self.prompt.take() {
            prompt.text_state.value().to_string()
        } else {
            String::new()
        }
    }

    pub fn set_has_unsaved_changes(&mut self, state: bool) {
        self.has_unsaved_changes = state;
    }

    pub fn render(&mut self, frame: &mut Frame, config: &Config) {
        let area = frame.area();
        // Reserve space for scrollbar and padding
        let content_width = area.width.saturating_sub(4).max(46);

        // Build sections with proper layout
        let sections = self.build_sections(&config.pomodoro);

        // Calculate total height: title (4) + spacing (1) + sections + padding (2)
        let sections_height: u16 = sections.iter().map(|s| s.height).sum();
        let total_height: u16 = 4 + 1 + sections_height + 2;

        // Create scroll view with full content size
        let mut scroll_view = ScrollView::new(Size::new(content_width, total_height))
            .vertical_scrollbar_visibility(ScrollbarVisibility::Automatic);

        // Render title at top
        let title_area = Rect::new(0, 0, content_width, 4);
        self.render_title(&mut scroll_view, title_area);

        // Render unsaved changes indicator in the spacing row between title and sections
        let indicator_area = Rect::new(0, 4, content_width, 1);
        self.render_unsaved_indicator(&mut scroll_view, indicator_area);

        // Render sections with proper spacing
        let mut y = 5u16; // Start after title + 1 row spacing
        for section in sections {
            let section_area = Rect::new(0, y, content_width, section.height);
            y += section.height;
            scroll_view.render_widget(section, section_area);
        }

        frame.render_stateful_widget(scroll_view, area, &mut self.scroll_state);

        // Render prompt popup
        self.render_prompt(frame, area);
    }

    fn render_title(&self, scroll_view: &mut ScrollView, area: Rect) {
        let big_text = BigText::builder()
            .pixel_size(PixelSize::Quadrant)
            .style(
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )
            .lines(vec!["Settings".into()])
            .centered()
            .build();
        scroll_view.render_widget(big_text, area);
    }

    fn render_unsaved_indicator(&self, scroll_view: &mut ScrollView, area: Rect) {
        if !self.has_unsaved_changes {
            return;
        }
        let line = Line::from(vec![Span::styled(
            "● Unsaved changes",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )]);
        scroll_view.render_widget(Paragraph::new(line), area);
    }

    fn render_prompt(&mut self, frame: &mut Frame, area: Rect) {
        // Render prompt overlay
        if let Some(ref mut prompt) = self.prompt {
            let popup_width = 50.min(area.width.saturating_sub(4));
            let popup_height = 3;

            let vertical = Layout::vertical([Constraint::Length(popup_height)]).flex(Flex::Center);
            let horizontal =
                Layout::horizontal([Constraint::Length(popup_width)]).flex(Flex::Center);
            let [popup_area] = vertical.areas(area);
            let [popup_area] = horizontal.areas(popup_area);

            frame.render_widget(Clear, popup_area);

            let block = Block::default()
                .title(prompt.label.clone())
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan));
            let inner = block.inner(popup_area);
            frame.render_widget(block, popup_area);

            TextPrompt::new(Cow::Borrowed("")).draw(frame, inner, &mut prompt.text_state);
        }
    }

    /// Build sections from config, calculating layout and identifying editable items
    fn build_sections(&self, config: &PomodoroConfig) -> Vec<Section> {
        let mut sections = Vec::new();
        let mut item_idx = 0u32;

        self.build_timer_section(&config.timer, &mut sections, &mut item_idx);
        self.build_hooks_section(&config.hook, &mut sections, &mut item_idx);
        self.build_alarm_section(&config.alarm, &mut sections, &mut item_idx);

        sections
    }

    fn build_timer_section(
        &self,
        config: &Timers,
        sections: &mut Vec<Section>,
        item_idx: &mut u32,
    ) {
        // Build Pomodoro Timer section
        let label = "󰔛 Pomodoro Timer";
        let color = SectionColor::from_label(label);
        let mut rows = Vec::new();

        // Durations subsection
        if !rows.is_empty() {
            rows.push(SectionRow::Blank);
        }
        rows.push(SectionRow::SubSectionHeader("Durations".to_string()));
        self.add_input_to_rows(
            "Focus",
            format!("{}", config.focus.as_secs() / 60),
            &mut rows,
            item_idx,
        );
        self.add_input_to_rows(
            "Short Break",
            format!("{}", config.short.as_secs() / 60),
            &mut rows,
            item_idx,
        );
        self.add_input_to_rows(
            "Long Break",
            format!("{}", config.long.as_secs() / 60),
            &mut rows,
            item_idx,
        );

        self.add_input_to_rows(
            "Long Break Interval",
            format!("{}", config.long_interval),
            &mut rows,
            item_idx,
        );

        // Auto Start subsection
        if !rows.is_empty() {
            rows.push(SectionRow::Blank);
        }
        rows.push(SectionRow::SubSectionHeader("Auto Start".to_string()));
        self.add_checkbox_to_rows("Focus", config.auto_focus, &mut rows, item_idx);
        self.add_checkbox_to_rows("Short Break", config.auto_short, &mut rows, item_idx);
        self.add_checkbox_to_rows("Long Break", config.auto_long, &mut rows, item_idx);

        let height = 2 + rows.iter().map(|r| r.height()).sum::<u16>();
        sections.push(Section {
            title: label.to_string(),
            color,
            height,
            rows,
        });
    }

    fn build_hooks_section(&self, config: &Hooks, sections: &mut Vec<Section>, item_idx: &mut u32) {
        // Build Command Hooks section
        let label = "󰛢 Command Hooks";
        let color = SectionColor::from_label(label);
        let mut rows = Vec::new();

        // Hooks subsection
        if !rows.is_empty() {
            rows.push(SectionRow::Blank);
        }
        rows.push(SectionRow::SubSectionHeader("Hooks".to_string()));
        self.add_input_to_rows("Focus", &config.focus, &mut rows, item_idx);
        self.add_input_to_rows("Short Break", &config.short, &mut rows, item_idx);
        self.add_input_to_rows("Long Break", &config.long, &mut rows, item_idx);

        let height = 2 + rows.iter().map(|r| r.height()).sum::<u16>();
        sections.push(Section {
            title: label.to_string(),
            color,
            height,
            rows,
        });
    }

    fn build_alarm_section(
        &self,
        config: &Alarms,
        sections: &mut Vec<Section>,
        item_idx: &mut u32,
    ) {
        let mut rows = Vec::new();

        // Alarm Files subsection
        rows.push(SectionRow::SubSectionHeader("Alarm Files".to_string()));
        self.add_input_to_rows(
            "Focus",
            get_alarm_path_value(&config.focus),
            &mut rows,
            item_idx,
        );
        self.add_input_to_rows(
            "Short Break",
            get_alarm_path_value(&config.short),
            &mut rows,
            item_idx,
        );
        self.add_input_to_rows(
            "Long Break",
            get_alarm_path_value(&config.long),
            &mut rows,
            item_idx,
        );

        // Alarm Volumes subsection
        rows.push(SectionRow::Blank);
        rows.push(SectionRow::SubSectionHeader("Alarm Volumes".to_string()));
        self.add_input_to_rows(
            "Focus",
            get_alarm_volume_value(&config.focus),
            &mut rows,
            item_idx,
        );
        self.add_input_to_rows(
            "Short Break",
            get_alarm_volume_value(&config.short),
            &mut rows,
            item_idx,
        );
        self.add_input_to_rows(
            "Long Break",
            get_alarm_volume_value(&config.long),
            &mut rows,
            item_idx,
        );

        let height = 2 + rows.iter().map(|r| r.height()).sum::<u16>();
        sections.push(Section {
            title: "󰕾 Alarm".to_string(),
            color: SectionColor::Alarm,
            height,
            rows,
        });
    }

    fn add_input_to_rows(
        &self,
        label: impl ToString,
        value: impl ToString,
        rows: &mut Vec<SectionRow>,
        item_idx: &mut u32,
    ) {
        let idx = *item_idx;
        *item_idx += 1;
        let is_selected = self.selected_idx == idx;
        rows.push(SectionRow::Input {
            label: label.to_string(),
            value: value.to_string(),
            is_selected,
        });
    }

    fn add_checkbox_to_rows(
        &self,
        label: &str,
        value: bool,
        rows: &mut Vec<SectionRow>,
        item_idx: &mut u32,
    ) {
        let idx = *item_idx;
        *item_idx += 1;
        let is_selected = self.selected_idx == idx;
        rows.push(SectionRow::Checkbox {
            label: label.to_string(),
            value,
            is_selected,
        });
    }
}

/// Represents a section with border
#[derive(Clone, Debug, PartialEq, Eq)]
struct Section {
    title: String,
    color: SectionColor,
    height: u16,
    rows: Vec<SectionRow>,
}

/// Individual row within a section
#[derive(Clone, Debug, PartialEq, Eq)]
enum SectionRow {
    Blank,
    SubSectionHeader(String),
    Input {
        label: String,
        value: String,
        is_selected: bool,
    },
    Checkbox {
        label: String,
        value: bool,
        is_selected: bool,
    },
}

impl SectionRow {
    fn height(&self) -> u16 {
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
        // Create block with border
        let block = Block::default()
            .title(self.title.clone())
            .title_style(self.color.title_style())
            .borders(Borders::ALL)
            .border_style(Style::default().fg(self.color.border_color()));

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
                let line = Line::from(Span::styled(
                    format!("▸ {} ", label),
                    Style::default()
                        .fg(Color::White)
                        .add_modifier(Modifier::BOLD)
                        .add_modifier(Modifier::UNDERLINED),
                ));
                Paragraph::new(line).render(area, buf);
            }
            SectionRow::Input {
                label,
                value,
                is_selected,
            } => {
                let bg = if is_selected {
                    Style::default().bg(Color::DarkGray)
                } else {
                    Style::default()
                };

                let line = Line::from(vec![
                    Span::styled(
                        format!("{}: ", label),
                        Style::default().add_modifier(Modifier::DIM).patch(bg),
                    ),
                    Span::styled(value, Style::default().patch(bg)),
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

                let checkbox = if value {
                    Span::styled("[x]", Style::default().fg(Color::Cyan).patch(bg))
                } else {
                    Span::styled("[ ]", Style::default().fg(Color::Cyan).patch(bg))
                };

                let line = Line::from(vec![
                    checkbox,
                    Span::styled(" ", bg),
                    Span::styled(label.clone(), bg),
                ]);
                Paragraph::new(line).render(area, buf);
            }
        }
    }
}

/// Section color scheme for visual distinction
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SectionColor {
    Timer,
    Hooks,
    Alarm,
}

impl SectionColor {
    fn border_color(self) -> Color {
        match self {
            SectionColor::Timer => Color::Cyan,
            SectionColor::Hooks => Color::Yellow,
            SectionColor::Alarm => Color::Magenta,
        }
    }

    fn title_style(self) -> Style {
        Style::default()
            .fg(self.border_color())
            .add_modifier(Modifier::BOLD)
    }

    fn from_label(label: impl AsRef<str>) -> Self {
        let label = label.as_ref();
        if label.contains("Timer") {
            SectionColor::Timer
        } else if label.contains("Hook") {
            SectionColor::Hooks
        } else if label.contains("Alarm") {
            SectionColor::Alarm
        } else {
            SectionColor::Timer
        }
    }
}

pub struct SettingsPrompt {
    pub text_state: TextState<'static>,
    pub label: String,
}

fn get_alarm_path_value(alarm: &Alarm) -> String {
    alarm
        .path
        .as_ref()
        .map(|p| p.display().to_string())
        .unwrap_or_default()
}

fn get_alarm_volume_value(alarm: &Alarm) -> String {
    alarm.volume.to_string()
}
