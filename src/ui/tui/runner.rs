use std::time::Duration;
use std::time::Instant;

use crossterm::event::Event;
use crossterm::event::KeyCode;
use crossterm::event::MouseEventKind;
use crossterm::event::{self};
use tui_widgets::prompts::State;
use tui_widgets::prompts::Status;

use crate::ui::UiError;
use crate::ui::core::Effect;
use crate::ui::core::Msg;
use crate::ui::core::Overlay;
use crate::ui::prelude::*;
use crate::ui::tui::TuiEffectHandler;
use crate::ui::tui::TuiError;
use crate::ui::tui::backend::Tui;
use crate::ui::tui::view::*;
use crate::ui::update::PomodoroMsg;
use crate::ui::update::TaskMsg;

/// The terminal UI implementation of the application runner using Crossterm and Ratatui.
pub struct TuiRunner {
    tick: TickTimer,
    redraw: bool,

    core: AppCore<TuiEffectHandler>,
    terminal: Tui,

    timer: TuiTimerView,
    settings: TuiSettingsView,
}

impl Runner for TuiRunner {
    fn run(&mut self) -> Result<(), UiError> {
        Ok(self.run_loop()?)
    }
}

macro_rules! dsp {
    // Direct core dispatch: dsp!(self, core, msg)
    ($self:ident, core, $msg:expr) => {{
        $self.core.dispatch($msg);
        $self.redraw = true;
    }};
    // Wrapped message dispatch: dsp!(self, pomo, msg) → Msg::Pomodoro(msg)
    ($self:ident, pomo,   $msg:expr) => {
        dsp!($self, core, Msg::Pomodoro($msg))
    };
    ($self:ident, router, $msg:expr) => {
        dsp!($self, core, Msg::Router($msg))
    };
    // View update dispatch: dsp!(self, timer, msg) → update → cmds
    ($self:ident, timer, $msg:expr) => {{
        let cmds = $self.timer.update($msg);
        for cmd in cmds {
            $self.core.dispatch(Msg::ViewTimerCmd(cmd));
        }
        $self.redraw = true;
    }};
    ($self:ident, setting, $msg:expr) => {{
        let cmds = $self.settings.update($msg);
        for cmd in cmds {
            $self.core.dispatch(Msg::ViewSettingsCmd(cmd));
        }
        $self.redraw = true;
    }};
}

impl TuiRunner {
    pub fn new(core: AppCore<TuiEffectHandler>) -> Result<Self, UiError> {
        let terminal = Tui::new()?;

        Ok(Self {
            core,
            terminal,
            timer: TuiTimerView::new(),
            settings: TuiSettingsView::new(),
            tick: TickTimer::default(),
            redraw: true,
        })
    }

    fn run_loop(&mut self) -> Result<(), TuiError> {
        self.initial();
        while !self.core.is_quit() {
            self.render_terminal()?;
            self.tick();
            self.handle_inputs()?;
        }
        Ok(())
    }

    fn initial(&mut self) {
        // Initial dispatch for auto-start.
        let auto_start = self.core.config().pomodoro.timer.auto_start_on_launch;

        if auto_start {
            dsp!(self, pomo, PomodoroMsg::Start);
        } else {
            dsp!(self, pomo, PomodoroMsg::StartPaused);
        }
    }

    fn render_terminal(&mut self) -> Result<(), TuiError> {
        if self.take_redraw() {
            self.terminal.draw(|f| {
                // Pages
                match self.core.router().active_page() {
                    Page::Timer => {
                        let is_prompting_transition =
                            self.core.overlay() == Some(Overlay::PromptingTransition);
                        self.timer.render(
                            f,
                            self.core.pomodoro(),
                            is_prompting_transition,
                            self.core.transition_prompt_at(),
                            self.core.current_task(),
                        )
                    }
                    Page::Settings => {
                        let is_config_dirty = self.core.is_config_dirty();
                        self.settings.render(f, self.core.config(), is_config_dirty)
                    }
                }

                let area = f.area();

                // Toast
                let toast = self.core.effects_mut().toast_mut();
                toast.set_area(area);
                f.render_widget(&**toast, area);

                if let Some(overlay) = self.core.overlay() {
                    match overlay {
                        Overlay::DuplicateWarning => f.render_widget(DuplicateWarning::new(), area),
                        Overlay::UnsavedWarning => f.render_widget(UnsavedWarning::new(), area),
                        Overlay::ResetWarning => f.render_widget(ResetWarning::new(), area),
                        Overlay::PromptingTransition => {} // Handled inside timer view for now
                    }
                }
            })?;
        }
        Ok(())
    }

    fn tick(&mut self) {
        if self.tick.new_tick() {
            self.core.effects_mut().toast_mut().tick();
            dsp!(self, core, Msg::Tick);
        }
    }

    // ---------------------------------------------------------
    //  ___ _  _ ___ _   _ _____
    // |_ _| \| | _ \ | | |_   _|
    //  | || .` |  _/ |_| | | |
    // |___|_|\_|_|  \___/  |_|
    // ---------------------------------------------------------

    fn handle_inputs(&mut self) -> Result<(), TuiError> {
        if event::poll(self.tick.time_until_next())? {
            while event::poll(Duration::ZERO)? {
                let event = event::read()?;
                #[cfg(target_os = "windows")]
                if let Event::Key(key) = &event {
                    if key.kind == event::KeyEventKind::Release {
                        continue;
                    }
                }

                if let Some(overlay) = self.core.overlay() {
                    match overlay {
                        Overlay::DuplicateWarning => {
                            self.handle_duplicate_warning(&event);
                            continue;
                        }
                        Overlay::UnsavedWarning => {
                            self.handle_unsaved_warning(&event);
                            continue;
                        }
                        Overlay::ResetWarning => {
                            self.handle_reset_warning(&event);
                            continue;
                        }
                        Overlay::PromptingTransition => {} // Handled below in timer
                    }
                }

                self.common_handler(&event);

                match self.core.router().active_page() {
                    Page::Settings => self.handle_settings(event),
                    Page::Timer => self.handle_timer(event),
                }
            }
        }
        Ok(())
    }

    fn common_handler(&mut self, event: &Event) {
        match event {
            Event::Key(key) => match key.code {
                KeyCode::Char('m') => self.core.execute_effect(Effect::StopSound),
                KeyCode::Char('q') => dsp!(self, core, Msg::Quit),
                _ => {}
            },
            Event::Resize(..) => self.redraw(),
            _ => {}
        }
    }

    fn handle_reset_warning(&mut self, event: &Event) {
        if let Event::Key(key) = event {
            use KeyCode as K;
            match key.code {
                K::Enter | K::Char('y') => dsp!(self, core, Msg::ResetWarningProceed),
                K::Esc | K::Char('n') => dsp!(self, core, Msg::ResetWarningCancel),
                _ => {}
            }
        }
    }

    fn handle_unsaved_warning(&mut self, event: &Event) {
        if let Event::Key(key) = event {
            use KeyCode as K;
            match key.code {
                K::Enter | K::Char('s') => dsp!(self, core, Msg::UnsavedWarningSave),
                K::Esc => dsp!(self, core, Msg::UnsavedWarningCancel),
                K::Char('q') => dsp!(self, core, Msg::UnsavedWarningQuit),
                _ => {}
            }
        }
    }

    fn handle_duplicate_warning(&mut self, event: &Event) {
        if let Event::Key(key) = event {
            use KeyCode as K;
            match key.code {
                K::Enter | K::Char('y') => dsp!(self, core, Msg::DuplicateWarningDismiss),
                K::Char('q') | K::Esc | K::Char('n') => dsp!(self, core, Msg::DuplicateWarningQuit),
                _ => {}
            }
        }
    }

    fn handle_timer(&mut self, event: Event) {
        if self.timer.is_editing() {
            return self.handle_timer_edit(event);
        }

        if self.core.overlay() == Some(Overlay::PromptingTransition) {
            return self.handle_timer_transition(event);
        }

        use KeyCode as K;
        use PomodoroMsg::*;
        if let Event::Key(key) = event {
            match key.code {
                K::Right | K::Char('l') => dsp!(self, pomo, Subtract(Duration::from_secs(30))),
                K::Down | K::Char('j') => dsp!(self, pomo, Subtract(Duration::from_secs(60))),
                K::Left | K::Char('h') => dsp!(self, pomo, Add(Duration::from_secs(30))),
                K::Up | K::Char('k') => dsp!(self, pomo, Add(Duration::from_secs(60))),
                K::Char(' ') => dsp!(self, pomo, TogglePause),
                K::Enter => dsp!(self, pomo, SkipSession),
                K::Backspace => dsp!(self, core, Msg::ResetWarningShow),
                K::Char('s') => dsp!(self, router, RouterMsg::GoTo(Page::Settings)),
                K::Char('t') => dsp!(self, timer, TimerMsg::StartTaskPrompt),
                K::Char('/') | K::Char('?') => dsp!(self, timer, TimerMsg::ToggleShowKeybinds),
                _ => {}
            }
        }
    }

    fn handle_timer_transition(&mut self, event: Event) {
        use KeyCode as K;
        if let Event::Key(key) = event {
            match key.code {
                K::Enter | K::Char('y') => {
                    dsp!(
                        self,
                        core,
                        Msg::ViewTimerCmd(TimerCmd::PromptTransitionAnsweredYes)
                    )
                }
                K::Esc | K::Char('n') => {
                    dsp!(
                        self,
                        core,
                        Msg::ViewTimerCmd(TimerCmd::PromptTransitionAnsweredNo)
                    )
                }
                _ => {}
            }
        }
    }

    fn handle_timer_edit(&mut self, event: Event) {
        if let Event::Key(key) = event
            && let Some(prompt) = self.timer.prompt_state_mut()
        {
            match key.code {
                KeyCode::Up | KeyCode::BackTab => {
                    if prompt.selected > 0 {
                        prompt.selected -= 1;
                    }
                    if let Some(task) = prompt.suggestions.get(prompt.selected) {
                        *prompt.text_state.value_mut() = task.name.clone();
                        let len = prompt.text_state.value().chars().count();
                        *prompt.text_state.position_mut() = len;
                    }
                    self.redraw = true;
                    return;
                }
                KeyCode::Down | KeyCode::Tab => {
                    if prompt.selected + 1 < prompt.suggestions.len() {
                        prompt.selected += 1;
                    }
                    if let Some(task) = prompt.suggestions.get(prompt.selected) {
                        *prompt.text_state.value_mut() = task.name.clone();
                        let len = prompt.text_state.value().chars().count();
                        *prompt.text_state.position_mut() = len;
                    }
                    self.redraw = true;
                    return;
                }
                KeyCode::Enter => {
                    if let Some(task) = prompt.suggestions.get(prompt.selected) {
                        let task = task.clone();
                        dsp!(self, timer, TimerMsg::CancelTaskPrompt);
                        dsp!(self, core, Msg::Task(TaskMsg::Select(task)));
                        return;
                    }
                }
                _ => {}
            }

            prompt.text_state.handle_key_event(key);
            self.redraw = true;

            if let Some(tasks) = self.core.task_suggestions() {
                let value = prompt.text_state.value();
                let prefix = value.to_lowercase();
                prompt.suggestions = if prefix.is_empty() {
                    Vec::new()
                } else {
                    tasks
                        .iter()
                        .filter(|t| t.name.to_lowercase().starts_with(&prefix))
                        .cloned()
                        .collect()
                };
                if prompt.selected >= prompt.suggestions.len() {
                    prompt.selected = prompt.suggestions.len().saturating_sub(1);
                }
            }

            match prompt.text_state.status() {
                Status::Done => {
                    let name = prompt.text_state.value().to_string();
                    log::debug!("task name: {}", name);
                    dsp!(self, timer, TimerMsg::CancelTaskPrompt);
                    dsp!(
                        self,
                        core,
                        Msg::Task(TaskMsg::Add {
                            name,
                            description: None,
                        })
                    );
                }
                Status::Aborted => dsp!(self, timer, TimerMsg::CancelTaskPrompt),
                _ => {}
            }
        }
    }

    fn handle_settings(&mut self, event: Event) {
        if self.settings.is_editing() {
            return self.handle_settings_edit(event);
        }

        use KeyCode as K;
        use SettingsMsg::*;
        match event {
            Event::Key(key) => match key.code {
                K::Up | K::Char('k') => dsp!(self, setting, SelectUp),
                K::BackTab => dsp!(self, setting, SectionPrev),
                K::Tab => dsp!(self, setting, SectionNext),
                K::Down | K::Char('j') => dsp!(self, setting, SelectDown),
                K::Enter | K::Char(' ') => {
                    let sel = self.settings.selected();
                    if sel.is_toggle() {
                        dsp!(self, setting, ApplyEdit(sel.into()));
                    } else {
                        dsp!(self, setting, StartEdit(&self.core.config().pomodoro));
                    }
                }
                K::Char('1') => dsp!(self, setting, SectionSelect(0)),
                K::Char('2') => dsp!(self, setting, SectionSelect(1)),
                K::Char('3') => dsp!(self, setting, SectionSelect(2)),
                K::Char('s') => dsp!(self, setting, SaveConfig),
                K::Char('c') | K::Char('y') => dsp!(self, setting, SelectForCopy),
                K::Char('v') | K::Char('p') => {
                    let pomo = &self.core.config().pomodoro;
                    dsp!(self, setting, CopyValue(pomo));
                }
                K::Esc => dsp!(self, router, RouterMsg::GoTo(Page::Timer)),
                K::Char('/') | K::Char('?') => dsp!(self, setting, ToggleShowKeybinds),
                _ => {}
            },
            Event::Mouse(mouse) => match mouse.kind {
                MouseEventKind::ScrollDown => dsp!(self, setting, ScrollDown),
                MouseEventKind::ScrollUp => dsp!(self, setting, ScrollUp),
                _ => {}
            },
            _ => {}
        }
    }

    fn handle_settings_edit(&mut self, event: Event) {
        if let Event::Key(key) = event {
            let is_path = self.settings.selected().is_path();

            if is_path {
                // Navigation for path fields
                if let Some(prompt) = self.settings.prompt_state_mut() {
                    match key.code {
                        KeyCode::Tab
                            if let Some(name) =
                                prompt.suggestions.get(prompt.suggested).cloned() =>
                        {
                            *prompt.text_state.value_mut() = name;
                            let len = prompt.text_state.value().chars().count();
                            *prompt.text_state.position_mut() = len;
                            self.settings.refresh_path_suggestions();
                            self.redraw();
                            return;
                        }
                        KeyCode::Up => {
                            if prompt.suggested > 0 {
                                prompt.suggested -= 1;
                            }
                            prompt.clamp_suggestion_scroll(MAX_VISIBLE_SUGGESTIONS);
                            self.redraw();
                            return;
                        }
                        KeyCode::Down => {
                            if prompt.suggested + 1 < prompt.suggestions.len() {
                                prompt.suggested += 1;
                            }
                            prompt.clamp_suggestion_scroll(MAX_VISIBLE_SUGGESTIONS);
                            self.redraw();
                            return;
                        }
                        _ => {}
                    }
                }
                // refresh after updat
                self.settings.refresh_path_suggestions();
            }
            if let Some(prompt) = self.settings.prompt_state_mut() {
                prompt.text_state.handle_key_event(key);
                self.redraw = true;

                match prompt.text_state.status() {
                    Status::Done => {
                        log::debug!("value {}", prompt.text_state.value());
                        dsp!(self, setting, SettingsMsg::SaveEdit);
                        dsp!(self, setting, SettingsMsg::CancelEditing);
                    }
                    Status::Aborted => dsp!(self, setting, SettingsMsg::CancelEditing),
                    _ => {}
                }
            }
        }
    }

    // ---------------------------------------------------------
    //  _  _ ___ _    ___ ___ ___  ___
    // | || | __| |  | _ \ __| _ \/ __|
    // | __ | _|| |__|  _/ _||   /\__ \
    // |_||_|___|____|_| |___|_|_\|___/
    // ---------------------------------------------------------

    fn redraw(&mut self) {
        self.redraw = true;
    }

    fn take_redraw(&mut self) -> bool {
        if self.redraw {
            self.redraw = false;
            true
        } else {
            false
        }
    }
}

// ---------------------------------------------------------
//  _____ ___ ___ _  __
// |_   _|_ _/ __| |/ /
//   | |  | | (__| ' <
//   |_| |___\___|_|\_\
// ---------------------------------------------------------

struct TickTimer {
    last_tick: Instant,
    tick_rate: Duration,
}

impl TickTimer {
    fn new(tick_rate: Duration) -> Self {
        Self {
            last_tick: Instant::now(),
            tick_rate,
        }
    }

    fn new_tick(&mut self) -> bool {
        let now = Instant::now();
        if now.duration_since(self.last_tick) >= self.tick_rate {
            self.last_tick = now;
            true
        } else {
            false
        }
    }

    fn time_until_next(&self) -> Duration {
        let elapsed = Instant::now().duration_since(self.last_tick);
        self.tick_rate.saturating_sub(elapsed)
    }
}

// ---------------------------------------------------------
//   ___ _____ _  _ ___ ___
//  / _ \_   _| || | __| _ \
// | (_) || | | __ | _||   /
//  \___/ |_| |_||_|___|_|_\
// ---------------------------------------------------------

impl From<SettingsItem> for ConfigMsg {
    fn from(value: SettingsItem) -> Self {
        use SettingsItem::*;
        match value {
            AutoStartOnLaunch => ConfigMsg::AutoStartOnLaunch,
            TimerAutoFocus => ConfigMsg::TimerAutoFocus,
            TimerAutoShort => ConfigMsg::TimerAutoShort,
            TimerAutoLong => ConfigMsg::TimerAutoLong,
            _ => unreachable!("toggle_config_msg called on non-toggle item"),
        }
    }
}

impl Default for TickTimer {
    fn default() -> Self {
        Self::new(Duration::from_secs(1))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tick_timer_new_tick_before_interval() {
        let mut timer = TickTimer::new(Duration::from_secs(60));
        assert!(!timer.new_tick());
    }

    #[test]
    fn tick_timer_new_tick_after_interval() {
        let mut timer = TickTimer::new(Duration::from_millis(10));
        std::thread::sleep(Duration::from_millis(15));
        assert!(timer.new_tick());
    }

    #[test]
    fn tick_timer_consecutive_calls() {
        let mut timer = TickTimer::new(Duration::from_millis(10));
        std::thread::sleep(Duration::from_millis(15));
        assert!(timer.new_tick());
        assert!(!timer.new_tick());
    }

    #[test]
    fn tick_timer_time_until_next_initial() {
        let timer = TickTimer::new(Duration::from_secs(5));
        let remaining = timer.time_until_next();

        assert!(remaining <= Duration::from_secs(5));
        assert!(remaining > Duration::from_secs(0));
    }

    #[test]
    fn tick_timer_time_until_next_saturates() {
        let timer = TickTimer::new(Duration::from_millis(10));
        std::thread::sleep(Duration::from_millis(20));

        // After the interval has passed, time_until_next should not underflow
        let remaining = timer.time_until_next();
        assert_eq!(remaining, Duration::ZERO);
    }

    #[test]
    fn settings_item_into_config_msg_toggle() {
        assert_eq!(
            ConfigMsg::from(SettingsItem::AutoStartOnLaunch),
            ConfigMsg::AutoStartOnLaunch
        );
        assert_eq!(
            ConfigMsg::from(SettingsItem::TimerAutoFocus),
            ConfigMsg::TimerAutoFocus
        );
        assert_eq!(
            ConfigMsg::from(SettingsItem::TimerAutoShort),
            ConfigMsg::TimerAutoShort
        );
        assert_eq!(
            ConfigMsg::from(SettingsItem::TimerAutoLong),
            ConfigMsg::TimerAutoLong
        );
    }

    #[test]
    #[should_panic(expected = "toggle_config_msg called on non-toggle item")]
    fn settings_item_into_config_msg_non_toggle_panics() {
        let _: ConfigMsg = SettingsItem::TimerFocus.into();
    }
}
