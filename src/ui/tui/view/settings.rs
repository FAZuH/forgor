use std::borrow::Cow;
use std::path::Path;
use std::path::PathBuf;
use std::sync::LazyLock;
use std::time::Duration;

use ratatui::layout::Flex;
use ratatui::prelude::*;
use ratatui::symbols::border;
use ratatui::widgets::Block;
use ratatui::widgets::Borders;
use ratatui::widgets::Clear;
use ratatui::widgets::Paragraph;
use strum::EnumCount;
use tui_widgets::prompts::FocusState;
use tui_widgets::prompts::State;
use tui_widgets::prompts::TextState;
use tui_widgets::prompts::prelude::*;
use tui_widgets::scrollview::ScrollView;
use tui_widgets::scrollview::ScrollViewState;
use tui_widgets::scrollview::ScrollbarVisibility;

use crate::config::Alarms;
use crate::config::Config;
use crate::config::Hooks;
use crate::config::Percentage;
use crate::config::PomodoroConfig;
use crate::config::Timers;
use crate::ui::prelude::*;

pub struct TuiSettingsView {
    selected: SettingsItem,
    select_for_copy: Option<SettingsItem>,
    scroll_state: ScrollViewState,
    prompt: Option<SettingsPrompt>,
    has_unsaved_changes: bool,
    show_keybinds: bool,
}

// ---------------------------------------------------------
//  ___ ___ _  _ ___  ___ ___
// | _ \ __| \| |   \| __| _ \
// |   / _|| .` | |) | _||   /
// |_|_\___|_|\_|___/|___|_|_\
// ---------------------------------------------------------

impl TuiSettingsView {
    pub fn render(&mut self, canvas: &mut Frame, state: &Config) {
        let area = canvas.area();
        let buf = canvas.buffer_mut();

        // Split area for scroll view and help bar
        let [content_area, help_area] =
            Layout::vertical([Constraint::Fill(1), Constraint::Length(3)]).areas(area);

        // Reserve space for scrollbar and padding
        let content_width = content_area.width.saturating_sub(2).max(46);

        // Build sections with proper layout
        let sections = self.sections(&state.pomodoro);

        // Create scroll view with full content size
        let mut scroll_view = ScrollView::new(Size::new(content_width, content_area.height))
            .vertical_scrollbar_visibility(ScrollbarVisibility::Automatic);

        // Render unsaved changes indicator in the spacing row between title and sections
        let indicator_area = Rect::new(0, 0, content_width, 1);
        self.save_indicator(&mut scroll_view, indicator_area);

        // Render sections with proper spacing
        let mut y = 2u16;
        let last = sections.last().unwrap().section;
        for section in sections {
            let section_area = if section.section == last {
                Rect::new(0, y, content_width, content_area.height)
            } else {
                Rect::new(0, y, content_width, section.height)
            };
            y += section.height;
            scroll_view.render_widget(section, section_area);
        }

        scroll_view.render(content_area, buf, self.scroll_state_mut());

        // Render help bar at bottom
        self.keybinds(help_area, buf, self.show_keybinds());

        // Render prompt popup (over full area, including help bar)
        self.prompt(canvas, area);
    }
}

impl TuiSettingsView {
    pub fn new() -> Self {
        Self {
            selected: SettingsItem::TimerFocus,
            select_for_copy: None,
            scroll_state: ScrollViewState::default(),
            prompt: None,
            has_unsaved_changes: false,
            show_keybinds: false,
        }
    }

    fn save_indicator(&self, scroll: &mut ScrollView, area: Rect) {
        if self.has_unsaved_changes() {
            scroll.render_widget(SAVED_INDICATOR.clone(), area);
        }
    }

    fn keybinds(&self, area: Rect, buf: &mut Buffer, show_keybinds: bool) {
        if show_keybinds {
            KEYBINDS_ON.clone().render(area, buf);
        } else {
            KEYBINDS_OFF.clone().render(area, buf);
        }
    }

    fn prompt(&mut self, frame: &mut Frame, area: Rect) {
        if let Some(ref mut prompt) = self.prompt_state_mut() {
            let buf = frame.buffer_mut();
            let popup_width = 50.min(area.width.saturating_sub(4));

            let inner_width = popup_width.saturating_sub(2).max(1);
            let text_len = prompt.text_state.value().chars().count() as u16;
            let prefix_len = 5; // "?  › "
            let lines_needed = (text_len + prefix_len) / inner_width + 1;
            let popup_height = (lines_needed + 2).max(3).min(area.height);
            let vertical = Layout::vertical([Constraint::Length(popup_height)]).flex(Flex::Center);
            let horizontal =
                Layout::horizontal([Constraint::Length(popup_width)]).flex(Flex::Center);
            let [popup_area] = vertical.areas(area);
            let [popup_area] = horizontal.areas(popup_area);

            Clear.render(popup_area, buf);

            let block = Block::default()
                .title(prompt.label.clone())
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan));
            let inner = block.inner(popup_area);
            block.render(popup_area, buf);

            TextPrompt::new(Cow::Borrowed("")).draw(frame, inner, &mut prompt.text_state);
        }
    }

    /// Build sections from config, calculating layout and identifying editable items
    fn sections(&self, config: &PomodoroConfig) -> Vec<Section> {
        let mut sections = Vec::new();

        self.timer_section(&config.timer, &mut sections);
        self.hook_section(&config.hook, &mut sections);
        self.alarm_section(&config.alarm, &mut sections);

        sections
    }

    fn timer_section(&self, conf: &Timers, sections: &mut Vec<Section>) {
        use SettingsItem::*;
        // Build Pomodoro Timer section
        let mut r = Vec::new();

        // Durations subsection
        if !r.is_empty() {
            r.push(SectionRow::Blank);
        }
        r.push(SectionRow::SubSectionHeader("Durations".into()));
        self.add_inpt(TimerFocus, format!("{}", conf.focus.as_secs() / 60), &mut r);
        self.add_inpt(TimerShort, format!("{}", conf.short.as_secs() / 60), &mut r);
        self.add_inpt(TimerLong, format!("{}", conf.long.as_secs() / 60), &mut r);
        self.add_inpt(TimerLongInterval, format!("{}", conf.long_interval), &mut r);

        // Auto Start subsection
        if !r.is_empty() {
            r.push(SectionRow::Blank);
        }
        r.push(SectionRow::SubSectionHeader("Auto Start".into()));
        self.add_box(AutoStartOnLaunch, conf.auto_start_on_launch, &mut r);
        self.add_box(TimerAutoFocus, conf.auto_focus, &mut r);
        self.add_box(TimerAutoShort, conf.auto_short, &mut r);
        self.add_box(TimerAutoLong, conf.auto_long, &mut r);

        let height = 2 + r.iter().map(|r| r.height()).sum::<u16>();
        sections.push(Section {
            title: "[1] Pomodoro Timer".into(),
            section: SettingsSection::Timer,
            sel_item: self.selected(),
            height,
            rows: r,
        });
    }

    fn hook_section(&self, conf: &Hooks, sections: &mut Vec<Section>) {
        use SettingsItem::*;
        // Build Command Hooks section
        let mut r = Vec::new();

        // Hooks subsection
        if !r.is_empty() {
            r.push(SectionRow::Blank);
        }
        r.push(SectionRow::SubSectionHeader("Hooks".into()));
        self.add_inpt(HookFocus, &conf.focus, &mut r);
        self.add_inpt(HookShort, &conf.short, &mut r);
        self.add_inpt(HookLong, &conf.long, &mut r);

        let height = 2 + r.iter().map(|r| r.height()).sum::<u16>();
        sections.push(Section {
            title: "[2] Command Hooks".into(),
            section: SettingsSection::Hook,
            sel_item: self.selected(),
            height,
            rows: r,
        });
    }

    fn alarm_section(&self, conf: &Alarms, sections: &mut Vec<Section>) {
        use SettingsItem::*;
        let mut r = Vec::new();

        // Alarm Files subsection
        r.push(SectionRow::SubSectionHeader("Alarm Files".into()));
        self.add_inpt(AlarmPathFocus, conf.focus.path(), &mut r);
        self.add_inpt(AlarmPathShort, conf.short.path(), &mut r);
        self.add_inpt(AlarmPathLong, conf.long.path(), &mut r);

        // Alarm Volumes subsection
        r.push(SectionRow::Blank);
        r.push(SectionRow::SubSectionHeader("Alarm Volumes".into()));
        self.add_inpt(AlarmVolumeFocus, conf.focus.volume(), &mut r);
        self.add_inpt(AlarmVolumeShort, conf.short.volume(), &mut r);
        self.add_inpt(AlarmVolumeLong, conf.long.volume(), &mut r);

        let height = 2 + r.iter().map(|r| r.height()).sum::<u16>();
        sections.push(Section {
            title: "[3] Alarm".into(),
            section: SettingsSection::Alarm,
            sel_item: self.selected(),
            height,
            rows: r,
        });
    }

    fn add_inpt(&self, item: SettingsItem, value: impl ToString, rows: &mut Vec<SectionRow>) {
        let value = value.to_string();
        rows.push(SectionRow::Input {
            label: item.label().into(),
            is_selected: self.selected() == item,
            warning: {
                if item.is_path() {
                    !Path::new(&value).exists()
                } else {
                    false
                }
            },
            value,
        });
    }

    fn add_box(&self, item: SettingsItem, value: bool, rows: &mut Vec<SectionRow>) {
        rows.push(SectionRow::Checkbox {
            label: item.label().into(),
            value,
            is_selected: self.selected() == item,
        });
    }
}

// ---------------------------------------------------------
//  _   _ ___ ___   _ _____ ___
// | | | | _ \   \ /_\_   _| __|
// | |_| |  _/ |) / _ \| | | _|
//  \___/|_| |___/_/ \_\_| |___|
// ---------------------------------------------------------

impl<'a> Updateable<SettingsMsg<'a>, SettingsCmd> for TuiSettingsView {
    fn update<'b>(&mut self, msg: SettingsMsg<'b>) -> Vec<SettingsCmd> {
        use SettingsMsg::*;
        let mut cmds = vec![];

        match msg {
            ApplyEdit(msg) => cmds.push(SettingsCmd::SaveEdit(msg)),
            CancelEditing => self.cancel_editing(),
            SaveConfig => cmds.push(SettingsCmd::SaveConfig),
            SaveEdit => cmds.extend(self.save_edit()),
            ScrollDown => self.scroll_down(),
            ScrollUp => self.scroll_up(),
            SectionNext => self.next_section(),
            SectionPrev => self.prev_section(),
            SectionSelect(idx) => self.select_section(SettingsSection::from_index(idx).unwrap()),
            SelectDown => self.select_down(),
            SelectUp => self.select_up(),
            SetShowKeybinds(v) => self.show_keybinds = v,
            SetUnsavedChanges(v) => self.has_unsaved_changes = v,
            ToggleShowKeybinds => self.toggle_keybinds(),
            StartEdit(config) => self.start_editing_for_field(config),
            CopyValue(config) => cmds.extend(self.copy_value(config)),
            SelectForCopy => self.select_for_copy = Some(self.selected),
        }

        cmds
    }
}

impl TuiSettingsView {
    pub fn prompt_state_mut(&mut self) -> Option<&mut SettingsPrompt> {
        self.prompt.as_mut()
    }

    pub fn scroll_state_mut(&mut self) -> &mut ScrollViewState {
        &mut self.scroll_state
    }

    pub fn has_unsaved_changes(&self) -> bool {
        self.has_unsaved_changes
    }

    /// Get current selection index
    pub fn selected(&self) -> SettingsItem {
        self.selected
    }

    /// Check if currently editing
    pub fn is_editing(&self) -> bool {
        self.prompt.is_some()
    }

    pub fn show_keybinds(&self) -> bool {
        self.show_keybinds
    }

    fn copy_value(&mut self, conf: &Config) -> Vec<SettingsCmd> {
        if let Some(from) = self.select_for_copy
            && let Some(value) = Self::field_value(from, &conf.pomodoro)
        {
            self.cmd_from_edit(value, self.selected)
                .unwrap_or_else(|e| vec![e])
        } else {
            Vec::new()
        }
    }

    fn field_value(item: SettingsItem, config: &PomodoroConfig) -> Option<String> {
        let alarm = &config.alarm;
        let hook = &config.hook;
        let timer = &config.timer;
        use SettingsItem::*;

        let mut value = match item {
            TimerFocus => format!("{}", timer.focus.as_secs() / 60),
            TimerShort => format!("{}", timer.short.as_secs() / 60),
            TimerLong => format!("{}", timer.long.as_secs() / 60),
            TimerLongInterval => format!("{}", timer.long_interval),
            HookFocus => hook.focus.clone(),
            HookShort => hook.short.clone(),
            HookLong => hook.long.clone(),
            AlarmPathFocus => alarm.focus.path(),
            AlarmPathShort => alarm.short.path(),
            AlarmPathLong => alarm.long.path(),
            AlarmVolumeFocus => alarm.focus.volume(),
            AlarmVolumeShort => alarm.short.volume(),
            AlarmVolumeLong => alarm.long.volume(),
            AutoStartOnLaunch | TimerAutoFocus | TimerAutoShort | TimerAutoLong => return None,
        };

        if item.is_percentage() {
            value = value[..value.len() - 1].to_string();
        }

        Some(value)
    }

    fn start_editing_for_field(&mut self, config: &PomodoroConfig) {
        let Some(value) = Self::field_value(self.selected, config) else {
            return;
        };

        let value_len = value.len();
        let mut text_state = TextState::new()
            .with_focus(FocusState::Focused)
            .with_value(value);
        *State::position_mut(&mut text_state) = value_len;

        self.prompt = Some(SettingsPrompt {
            text_state,
            label: self.selected().to_string(),
        });
    }

    fn cmd_from_edit(
        &mut self,
        value: String,
        selected: SettingsItem,
    ) -> Result<Vec<SettingsCmd>, SettingsCmd> {
        use ConfigMsg as C;
        use SettingsItem as I;

        let mut cmds: Vec<SettingsCmd> = vec![];

        let conf_msg = match selected {
            I::AutoStartOnLaunch => C::AutoStartOnLaunch,
            I::TimerFocus => C::TimerFocus(self.parse_dur(value)?),
            I::TimerShort => C::TimerShort(self.parse_dur(value)?),
            I::TimerLong => C::TimerLong(self.parse_dur(value)?),
            I::TimerLongInterval => {
                C::TimerLongInterval(self.try_parse(value, |s| s.parse::<u32>(), "integer")?)
            }
            I::TimerAutoFocus => C::TimerAutoFocus,
            I::TimerAutoShort => C::TimerAutoShort,
            I::TimerAutoLong => C::TimerAutoLong,
            I::HookFocus => C::HookFocus(value),
            I::HookShort => C::HookShort(value),
            I::HookLong => C::HookLong(value),
            I::AlarmPathFocus => C::AlarmPathFocus(self.parse_path(value, &mut cmds)),
            I::AlarmPathShort => C::AlarmPathShort(self.parse_path(value, &mut cmds)),
            I::AlarmPathLong => C::AlarmPathLong(self.parse_path(value, &mut cmds)),
            I::AlarmVolumeFocus => C::AlarmVolumeFocus(self.parse_vol(value)?),
            I::AlarmVolumeShort => C::AlarmVolumeShort(self.parse_vol(value)?),
            I::AlarmVolumeLong => C::AlarmVolumeLong(self.parse_vol(value)?),
        };

        cmds.push(SettingsCmd::SaveEdit(conf_msg));
        Ok(cmds)
    }

    fn toggle_keybinds(&mut self) {
        self.update(SettingsMsg::SetShowKeybinds(!self.show_keybinds));
    }

    /// Select item up
    fn select_up(&mut self) {
        let idx = self
            .selected
            .index()
            .saturating_sub(1)
            .clamp(0, SettingsItem::COUNT as u32 - 1); // 13 items total
        self.selected = SettingsItem::from_index(idx).unwrap();
    }

    /// Select item down
    fn select_down(&mut self) {
        let idx = self
            .selected
            .index()
            .saturating_add(1)
            .clamp(0, SettingsItem::COUNT as u32 - 1); // 13 items total
        self.selected = SettingsItem::from_index(idx).unwrap();
    }

    fn prev_section(&mut self) {
        let idx = (self.selected.section().index() + SettingsItem::COUNT as u32 - 1)
            % SettingsItem::COUNT as u32;
        self.selected =
            SettingsItem::from_index(SettingsSection::from_index(idx).unwrap().item_begin_idx())
                .unwrap();
    }

    fn next_section(&mut self) {
        let idx = (self.selected.section().index() + 1) % SettingsItem::COUNT as u32;
        self.selected =
            SettingsItem::from_index(SettingsSection::from_index(idx).unwrap().item_begin_idx())
                .unwrap();
    }

    fn select_section(&mut self, section: SettingsSection) {
        self.selected = SettingsItem::from_index(section.item_begin_idx()).unwrap();
    }

    /// Scroll up by one row
    fn scroll_up(&mut self) {
        self.scroll_state.scroll_up();
    }

    /// Scroll down by one row
    fn scroll_down(&mut self) {
        self.scroll_state.scroll_down();
    }

    /// Cancel editing
    fn cancel_editing(&mut self) {
        self.prompt = None;
    }

    fn save_edit(&mut self) -> Vec<SettingsCmd> {
        let value = self
            .prompt
            .take()
            .map(|v| v.text_state.value().to_string())
            .unwrap_or_default();
        log::debug!("addr settings {:p}", self);
        log::debug!("value {}", value);
        self.update(SettingsMsg::CancelEditing);
        self.cmd_from_edit(value, self.selected)
            .unwrap_or_else(|e| vec![e])
    }

    fn parse_path(&mut self, s: impl AsRef<str>, cmds: &mut Vec<SettingsCmd>) -> Option<PathBuf> {
        let s = s.as_ref();
        if s.is_empty() {
            return None;
        }
        let path = PathBuf::from(s);
        if !path.exists() {
            cmds.push(SettingsCmd::ShowToast {
                message: "Path does not exist".to_string(),
                r#type: ToastType::Warning,
            });
        }
        Some(path)
    }

    fn parse_dur(&self, s: impl AsRef<str>) -> Result<Duration, SettingsCmd> {
        self.try_parse(s, |s| s.parse::<u64>(), "integer")
            .map(|val| Duration::from_secs(val * 60))
    }

    fn parse_vol(&self, s: impl AsRef<str>) -> Result<Percentage, SettingsCmd> {
        let s = s.as_ref();
        if s.is_empty() {
            Ok(Percentage::default())
        } else {
            self.try_parse(s, |s| Percentage::try_from(s), "percent")
        }
    }

    fn try_parse<T, E: std::fmt::Debug>(
        &self,
        s: impl AsRef<str>,
        f: impl for<'a> FnOnce(&'a str) -> Result<T, E>,
        label: &str,
    ) -> Result<T, SettingsCmd> {
        let s = s.as_ref();
        f(s).map_err(|e| SettingsCmd::ShowToast {
            message: format!("Failed converting '{s}' to {label}: {e:?}"),
            r#type: ToastType::Error,
        })
    }
}

// ---------------------------------------------------------
//  _  _ ___ _    ___ ___ ___
// | || | __| |  | _ \ __| _ \
// | __ | _|| |__|  _/ _||   /
// |_||_|___|____|_| |___|_|_\
// ---------------------------------------------------------

pub struct SettingsPrompt {
    pub text_state: TextState<'static>,
    pub label: String,
}

/// Represents a section with border
#[derive(Clone, Debug, PartialEq, Eq)]
struct Section {
    title: String,
    section: SettingsSection,
    sel_item: SettingsItem,
    height: u16,
    rows: Vec<SectionRow>,
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
enum SectionRow {
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

static KEYBINDS_ON: LazyLock<Paragraph<'static>> = LazyLock::new(|| {
    let dim = Style::default().dim();
    let bright = Style::default();
    let sep = Span::styled(" • ", dim);
    Paragraph::new(vec![
        Line::from(vec![
            Span::styled("Space/Enter", bright),
            Span::styled(": Edit", dim),
            sep.clone(),
            Span::styled("s", bright),
            Span::styled(": Save", dim),
            sep.clone(),
            Span::styled("c/y", bright),
            Span::styled(": Copy", dim),
            sep.clone(),
            Span::styled("v/p", bright),
            Span::styled(": Paste", dim),
        ]),
        Line::from(vec![
            Span::styled("↑/↓/j/k", bright),
            Span::styled(": Navigate", dim),
            sep.clone(),
            Span::styled("Tab", bright),
            Span::styled(": Sections", dim),
            sep.clone(),
            Span::styled("1/2/3", bright),
            Span::styled(": Jump", dim),
            sep.clone(),
            Span::styled("Esc", bright),
            Span::styled(": Back", dim),
            sep.clone(),
            Span::styled("q", bright),
            Span::styled(": Quit", dim),
            sep.clone(),
            Span::styled("m", bright),
            Span::styled(": Stop alarm", dim),
            sep.clone(),
            Span::styled("?", bright),
            Span::styled(": Disable Help", dim),
        ]),
    ])
    .alignment(Alignment::Center)
});

static KEYBINDS_OFF: LazyLock<Paragraph<'static>> = LazyLock::new(|| {
    let dim = Style::default().dim();
    let bright = Style::default();
    let line = Line::from(vec![Span::styled("?", bright), Span::styled(": Help", dim)]);
    Paragraph::new(line).alignment(Alignment::Center)
});

static SAVED_INDICATOR: LazyLock<Paragraph<'static>> = LazyLock::new(|| {
    Paragraph::new(Line::from(vec![Span::styled(
        "● Unsaved changes",
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD),
    )]))
});

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn item_idx() {
        use SettingsItem::*;
        use SettingsSection::*;
        assert_eq!(TimerFocus.section(), Timer);
        assert_eq!(TimerAutoLong.section(), Timer);

        assert_eq!(HookFocus.section(), Hook);
        assert_eq!(HookLong.section(), Hook);

        assert_eq!(AlarmPathFocus.section(), Alarm);
        assert_eq!(AlarmVolumeLong.section(), Alarm);
    }
}
