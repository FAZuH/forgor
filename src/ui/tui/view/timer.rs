use std::borrow::Cow;
use std::collections::HashMap;
use std::sync::LazyLock;
use std::time::Duration;
use std::time::Instant;

use ratatui::layout::Flex;
use ratatui::prelude::*;
use ratatui::symbols::border;
use ratatui::widgets::Block;
use ratatui::widgets::Borders;
use ratatui::widgets::Clear;
use ratatui::widgets::Gauge;
use ratatui::widgets::Paragraph;
use tui_widgets::popup::Popup;
use tui_widgets::prompts::FocusState;
use tui_widgets::prompts::State;
use tui_widgets::prompts::TextState;
use tui_widgets::prompts::prelude::*;

use crate::model::Mode;
use crate::model::Pomodoro;
use crate::model::Task;
use crate::ui::prelude::*;
use crate::utils;

/// Renders the Pomodoro timer interface and manages view-local state.
pub struct TuiTimerView {
    show_keybinds: bool,
    prompt: Option<TimerPrompt>,
}

pub struct TimerPrompt {
    pub text_state: TextState<'static>,
    pub suggestions: Vec<Task>,
    pub selected: usize,
}

impl TuiTimerView {
    pub fn new() -> Self {
        Self {
            show_keybinds: false,
            prompt: None,
        }
    }

    pub fn render(
        &mut self,
        canvas: &mut Frame,
        state: &Pomodoro,
        is_prompting_transition: bool,
        current_task: Option<&Task>,
    ) {
        let area = canvas.area();
        let buf = canvas.buffer_mut();

        let show_binds = self.show_keybinds();

        let mode = state.mode();
        let remaining = state.remaining_time();
        let paused = !state.is_running();
        let progress = state.progress();

        let rows = LAYOUT.split(area);

        self.paused_time(rows[1], buf, state.paused_at());
        self.state(rows[3], buf, mode, paused);
        self.timer(rows[5], buf, &remaining, mode);
        self.progress_bar(rows[6], buf, progress, mode);
        self.stats(rows[8], buf, state);
        if let Some(task) = current_task {
            self.task(rows[10], buf, task);
        }
        self.keybinds(rows[12], buf, show_binds);

        self.prompt_transition(area, buf, state, is_prompting_transition);
        self.prompt_task(canvas, area);
    }
}

impl TuiTimerView {
    // Render popup if prompt is active
    fn prompt_transition(
        &self,
        area: Rect,
        buf: &mut Buffer,
        pomo: &Pomodoro,
        is_prompting_transition: bool,
    ) {
        if is_prompting_transition {
            let next = pomo.next_mode().to_string().to_lowercase();
            let body = Text::from(vec![
                Line::from(""),
                Line::from(""),
                Line::from(format!("     start {next} session?     ")).alignment(Alignment::Center),
                Line::from(""),
                Line::from(""),
                Line::from("              ").alignment(Alignment::Center),
                Line::from(vec![
                    Span::from("       "),
                    Span::styled(
                        "  y/Enter: Yes  ",
                        Style::new().fg(Color::Green).bg(Color::DarkGray),
                    ),
                    Span::from("   "),
                    Span::styled(
                        "  n/Esc: No  ",
                        Style::new().fg(Color::Red).bg(Color::DarkGray),
                    ),
                    Span::from("       "),
                ])
                .alignment(Alignment::Center),
                Line::from(""),
                Line::from(""),
            ])
            .alignment(Alignment::Center);

            Widget::render(
                Popup::new(body)
                    .border_style(Style::new().fg(Color::Yellow))
                    .border_set(border::ROUNDED),
                area,
                buf,
            );
        }
    }

    fn prompt_task(&mut self, frame: &mut Frame, area: Rect) {
        if let Some(ref mut prompt) = self.prompt {
            let popup_width = 40.min(area.width.saturating_sub(4));
            let inner_width = popup_width.saturating_sub(2).max(1);
            let text_len = prompt.text_state.value().chars().count() as u16;
            let prefix_len = 5;
            let lines_needed = (text_len + prefix_len) / inner_width + 1;
            let prompt_input_height = lines_needed;

            let max_visible = 5;
            let num_suggestions = (prompt.suggestions.len() as u16).min(max_visible);
            let suggestions_height = if prompt.suggestions.is_empty() {
                0
            } else {
                num_suggestions + 1 // +1 separator line
            };
            let popup_height = (prompt_input_height + suggestions_height + 2).min(area.height);

            let vertical = Layout::vertical([Constraint::Length(popup_height)]).flex(Flex::Center);
            let horizontal =
                Layout::horizontal([Constraint::Length(popup_width)]).flex(Flex::Center);
            let [popup_area] = vertical.areas(area);
            let [popup_area] = horizontal.areas(popup_area);

            let buf = frame.buffer_mut();
            Clear.render(popup_area, buf);

            let block = Block::default()
                .title(" New task ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan));
            let inner = block.inner(popup_area);
            block.render(popup_area, buf);

            if prompt.suggestions.is_empty() {
                TextPrompt::new(Cow::Borrowed("")).draw(frame, inner, &mut prompt.text_state);
                return;
            }

            let [prompt_area, rest_area] =
                Layout::vertical([Constraint::Length(prompt_input_height), Constraint::Fill(1)])
                    .areas(inner);

            let [sep_area, list_area] =
                Layout::vertical([Constraint::Length(1), Constraint::Fill(1)]).areas(rest_area);

            let sep = Line::from("─".repeat(inner_width as usize))
                .style(Style::default().fg(Color::DarkGray));
            Paragraph::new(sep).render(sep_area, buf);

            let selected_style = Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::REVERSED);
            let dim = Style::default().dim();

            let suggestion_lines: Vec<Line> = prompt
                .suggestions
                .iter()
                .enumerate()
                .take(max_visible as usize)
                .map(|(idx, task)| {
                    let style = if idx == prompt.selected {
                        selected_style
                    } else {
                        dim
                    };
                    Line::styled(format!(" {name}", name = task.name), style)
                })
                .collect();

            Paragraph::new(suggestion_lines).render(list_area, buf);

            TextPrompt::new(Cow::Borrowed("")).draw(frame, prompt_area, &mut prompt.text_state);
        }
    }

    fn paused_time(&self, area: Rect, buf: &mut Buffer, paused_time: Option<Instant>) {
        if let Some(time) = paused_time {
            let dim = Style::default().dim();
            let total_secs = Instant::now().duration_since(time).as_secs();
            let minutes = total_secs / 60;
            let seconds = total_secs % 60;
            let formatted_time = if minutes == 0 {
                format!("{seconds}s")
            } else {
                format!("{minutes}m {seconds}s")
            };

            let line = Line::styled(format!("Paused {formatted_time} ago"), dim);

            Paragraph::new(line)
                .alignment(HorizontalAlignment::Center)
                .render(area, buf);
        }
    }

    fn state(&self, area: Rect, buf: &mut Buffer, mode: Mode, paused: bool) {
        let (label, label_width) = &STATE_LABELS[&mode];
        let color = mode.into();
        let center = Alignment::Center;

        if paused {
            let [area_label, area_paused] = Layout::horizontal([
                Constraint::Length(*label_width),
                Constraint::Length(*PAUSED_WIDTH),
            ])
            .flex(Flex::Center)
            .areas::<2>(area);

            Paragraph::new(label.as_str())
                .style(Style::default().fg(color).add_modifier(Modifier::BOLD))
                .alignment(center)
                .render(area_label, buf);
            PAUSED_PARAGRAPH.clone().render(area_paused, buf);
        } else {
            Paragraph::new(label.as_str())
                .style(Style::default().fg(color))
                .alignment(center)
                .render(area, buf);
        }
    }

    fn timer(&self, area: Rect, buf: &mut Buffer, remaining: &Duration, color: impl Into<Color>) {
        let time_str = format_duration_clock(remaining);
        let ascii = utils::ascii_mono12(&time_str);
        let width = utils::string_width(&ascii) as u16;
        let height = utils::string_height(&ascii) as u16;
        let area = area.centered(Constraint::Length(width), Constraint::Length(height));

        Paragraph::new(ascii)
            .style(
                Style::default()
                    .fg(color.into())
                    .add_modifier(Modifier::BOLD),
            )
            .alignment(Alignment::Center)
            .render(area, buf);
    }

    fn progress_bar(&self, area: Rect, buf: &mut Buffer, progress: f64, color: impl Into<Color>) {
        let layout = Layout::horizontal([Constraint::Length(55)]).flex(Flex::Center);
        let area = area.layout::<1>(&layout)[0];

        Gauge::default()
            .ratio(progress.clamp(0.0, 1.0))
            .use_unicode(true)
            .gauge_style(Style::default().fg(color.into()))
            .render(area, buf);
    }

    fn stats(&self, area: Rect, buf: &mut Buffer, state: &Pomodoro) {
        let long_interval = state.long_interval();
        let total_sessions = state.total_sessions();
        let focus_sessions = state.focus_sessions();
        let before_long_break = state.before_long_break();
        let dim = Style::default().dim();
        let bright = Style::default();
        let line = Line::from(vec![
            Span::styled("Focused: ", dim),
            Span::styled(focus_sessions.to_string(), bright),
            Span::styled("  │  Sessions: ", dim),
            Span::styled(total_sessions.to_string(), bright),
            Span::styled("  │  Long break every: ", dim),
            Span::styled(long_interval.to_string(), bright),
            Span::styled(format!(" (in {before_long_break} focus sessions)"), dim),
        ]);

        Paragraph::new(line)
            .alignment(Alignment::Center)
            .render(area, buf);
    }

    fn task(&self, area: Rect, buf: &mut Buffer, state: &Task) {
        let line = Line::from(vec![
            Span::styled("Current task: ", Style::default().dim()),
            Span::styled(&state.name, Style::default()),
        ]);

        Paragraph::new(line)
            .alignment(Alignment::Center)
            .render(area, buf);
    }

    fn keybinds(&self, area: Rect, buf: &mut Buffer, show: bool) {
        if show {
            KEYBINDS_ON.clone().render(area, buf);
        } else {
            KEYBINDS_OFF.clone().render(area, buf);
        }
    }
}

fn format_duration_clock(d: &Duration) -> String {
    let secs = d.as_secs();
    format!("{:02}:{:02}", secs / 60, secs % 60)
}

impl From<Mode> for Color {
    fn from(value: Mode) -> Self {
        match value {
            Mode::Focus => Color::LightBlue,
            Mode::ShortBreak => Color::LightGreen,
            Mode::LongBreak => Color::LightCyan,
        }
    }
}

static LAYOUT: LazyLock<Layout> = LazyLock::new(|| {
    Layout::vertical([
        Constraint::Fill(1),
        Constraint::Length(1), // paused_time
        Constraint::Length(1),
        Constraint::Length(3), // state
        Constraint::Length(2),
        Constraint::Length(9), // timer
        Constraint::Length(1), // progress_bar
        Constraint::Length(1),
        Constraint::Length(1), // stats
        Constraint::Length(1),
        Constraint::Length(1), // task
        Constraint::Length(1),
        Constraint::Length(3), // keybinds
        Constraint::Fill(1),
    ])
});

static STATE_LABELS: LazyLock<HashMap<Mode, (String, u16)>> = LazyLock::new(|| {
    let mut ret = HashMap::new();
    for (state, text) in [
        (Mode::Focus, "FOCUS"),
        (Mode::ShortBreak, "SHORT BREAK"),
        (Mode::LongBreak, "LONG BREAK"),
    ] {
        let label = utils::ascii_future(text);
        let width = utils::string_width(&label) as u16;
        ret.insert(state, (label, width));
    }

    ret
});

static PAUSED_TEXT: LazyLock<String> = LazyLock::new(|| utils::ascii_future(" ( PAUSED )"));
static PAUSED_WIDTH: LazyLock<u16> =
    LazyLock::new(|| utils::string_width(PAUSED_TEXT.as_str()) as u16);
static PAUSED_PARAGRAPH: LazyLock<Paragraph<'static>> = LazyLock::new(|| {
    Paragraph::new(PAUSED_TEXT.to_string()).style(
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD),
    )
});

static KEYBINDS_ON: LazyLock<Paragraph<'static>> = LazyLock::new(|| {
    let dim = Style::default().dim();
    let bright = Style::default();
    let sep = Span::styled(" • ", dim);
    Paragraph::new(vec![
        Line::from(vec![
            Span::styled("Space", bright),
            Span::styled(": Pause", dim),
            sep.clone(),
            Span::styled("Enter", bright),
            Span::styled(": Skip", dim),
            sep.clone(),
            Span::styled("Backspace", bright),
            Span::styled(": Reset", dim),
            sep.clone(),
            Span::styled("←/→/h/l", bright),
            Span::styled(": ±30s", dim),
            sep.clone(),
            Span::styled("↑/↓/j/k", bright),
            Span::styled(": ±1m", dim),
            sep.clone(),
            Span::styled("q", bright),
            Span::styled(": Quit", dim),
        ]),
        Line::from(vec![
            Span::styled("m", bright),
            Span::styled(": Stop alarm", dim),
            sep.clone(),
            Span::styled("s", bright),
            Span::styled(": Settings", dim),
            sep.clone(),
            Span::styled("t", bright),
            Span::styled(": Add task", dim),
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

impl Updateable<TimerMsg, TimerCmd> for TuiTimerView {
    fn update(&mut self, msg: TimerMsg) -> Vec<TimerCmd> {
        use TimerMsg::*;
        let ret = vec![];
        match msg {
            SetShowKeybinds(v) => self.show_keybinds = v,
            ToggleShowKeybinds => {
                let new = !self.show_keybinds;
                self.update(TimerMsg::SetShowKeybinds(new));
            }
            StartTaskPrompt => {
                let text_state = TextState::new().with_focus(FocusState::Focused);
                self.prompt = Some(TimerPrompt {
                    text_state,
                    suggestions: Vec::new(),
                    selected: 0,
                });
                return vec![TimerCmd::FetchAllTasks];
            }
            CancelTaskPrompt => {
                self.prompt = None;
            }
        }

        ret
    }
}

impl TuiTimerView {
    pub fn show_keybinds(&self) -> bool {
        self.show_keybinds
    }

    pub fn is_editing(&self) -> bool {
        self.prompt.is_some()
    }

    pub fn prompt_state_mut(&mut self) -> Option<&mut TimerPrompt> {
        self.prompt.as_mut()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::update::TimerCmd;

    #[test]
    fn start_task_prompt_emits_fetch_all() {
        let mut view = TuiTimerView::new();

        let cmds = view.update(TimerMsg::StartTaskPrompt);

        assert!(view.prompt.is_some());
        assert_eq!(cmds.len(), 1);
        assert!(matches!(cmds[0], TimerCmd::FetchAllTasks));
    }

    #[test]
    fn cancel_task_prompt_clears_prompt() {
        let mut view = TuiTimerView::new();
        view.update(TimerMsg::StartTaskPrompt);
        assert!(view.prompt.is_some());

        let cmds = view.update(TimerMsg::CancelTaskPrompt);

        assert!(view.prompt.is_none());
        assert!(cmds.is_empty());
    }
}
