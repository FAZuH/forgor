use std::time::Duration;
use std::time::Instant;

use crossterm::event::Event;
use crossterm::event::KeyCode;
use crossterm::event::MouseEventKind;
use crossterm::event::{self};
use tui_widgets::prompts::State;
use tui_widgets::prompts::Status;

use crate::ui::UiError;
use crate::ui::core::Cmd;
use crate::ui::core::Msg;
use crate::ui::prelude::*;
use crate::ui::runtime::Runtime;
use crate::ui::tui::TuiEffectHandler;
use crate::ui::tui::TuiError;
use crate::ui::tui::backend::Tui;
use crate::ui::tui::view::*;
use crate::ui::update::PomodoroMsg;

pub struct TuiRunner {
    tick: TickTimer,
    redraw: bool,

    runtime: Runtime<TuiEffectHandler>,
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
    ($self:ident, pomo, $msg:expr) => {{
        log::trace!("msg config: {:?}", $msg);
        $self.runtime.dispatch(Msg::Pomodoro($msg));
        $self.redraw = true;
    }};
    ($self:ident, config, $msg:expr) => {{
        log::trace!("msg config: {:?}", $msg);
        $self.runtime.dispatch(Msg::Config($msg));
        $self.redraw = true;
    }};
    ($self:ident, timer, $msg:expr) => {{
        log::trace!("msg timer: {:?}", $msg);
        let cmds = $self.timer.update($msg);
        for cmd in cmds {
            $self.runtime.dispatch(Msg::ViewTimerCmd(cmd));
        }
        $self.redraw = true;
    }};
    ($self:ident, setting, $msg:expr) => {{
        let cmds = $self.settings.update($msg);
        for cmd in cmds {
            $self.runtime.dispatch(Msg::ViewSettingsCmd(cmd));
        }
        $self.redraw = true;
    }};
    ($self:ident, runtime, $page:expr) => {{
        $self.runtime.core_mut().router_mut().navigate($page);
        $self.redraw = true;
    }};
}

impl TuiRunner {
    pub fn new(runtime: Runtime<TuiEffectHandler>) -> Result<Self, UiError> {
        let terminal = Tui::new()?;
        Ok(Self {
            runtime,
            terminal,
            timer: TuiTimerView::new(),
            settings: TuiSettingsView::new(),
            tick: TickTimer::default(),
            redraw: true,
        })
    }

    fn run_loop(&mut self) -> Result<(), TuiError> {
        // Initial dispatch for auto-start.
        let auto_start = self
            .runtime
            .core()
            .config()
            .pomodoro
            .timer
            .auto_start_on_launch;
        if auto_start {
            dsp!(self, pomo, PomodoroMsg::Start);
        } else {
            dsp!(self, pomo, PomodoroMsg::StartPaused);
        }

        while !self.runtime.core().router().is_quit() {
            self.render_terminal()?;
            self.tick();
            self.handle_inputs()?;
        }
        Ok(())
    }

    fn render_terminal(&mut self) -> Result<(), TuiError> {
        if self.take_redraw() {
            self.sync_views();

            let (core, effects) = self.runtime.split_mut();

            self.terminal.draw(|f| {
                match core.router().active_page() {
                    Some(Page::Timer) => self.timer.render(f, core.pomodoro()),
                    Some(Page::Settings) => self.settings.render(f, core.config()),
                    None => {}
                }

                let toast = effects.toast_mut();
                toast.set_area(f.area());
                f.render_widget(&**toast, f.area());
            })?;
        }
        Ok(())
    }

    fn tick(&mut self) {
        if self.tick.new_tick() {
            self.runtime.effects_mut().toast_mut().tick();
            self.runtime.dispatch(Msg::Tick);
            self.redraw();
        }
    }

    fn sync_views(&mut self) {
        let core = self.runtime.core();

        let is_trans = core.is_prompting_transition();
        let changes = core.is_config_dirty();

        dsp!(self, timer, TimerMsg::SetPromptTransition(is_trans));
        dsp!(self, setting, SettingsMsg::SetUnsavedChanges(changes));
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
                self.common_handler(&event);
                match self.runtime.core().router().active_page() {
                    Some(Page::Settings) => self.handle_settings(event),
                    Some(Page::Timer) => self.handle_timer(event),
                    None => self.runtime.dispatch(Msg::Quit),
                }
            }
        }
        Ok(())
    }

    fn common_handler(&mut self, event: &Event) {
        if let Event::Key(key) = event
            && let KeyCode::Char('m') = key.code
        {
            self.runtime.execute_effect(Cmd::StopSound)
        }
    }

    fn handle_timer(&mut self, event: Event) {
        if self.timer.prompt_transition() {
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
                K::Backspace => dsp!(self, pomo, ResetSession),
                K::Char('q') => self.runtime.dispatch(Msg::Quit),
                K::Char('s') => dsp!(self, runtime, Page::Settings),
                K::Char('/') | K::Char('?') => dsp!(self, timer, TimerMsg::ToggleShowKeybinds),
                _ => {}
            }
        }
    }

    fn handle_timer_transition(&mut self, event: Event) {
        use KeyCode as K;
        use TimerMsg::*;
        if let Event::Key(key) = event {
            match key.code {
                K::Enter | K::Char('y') => dsp!(self, timer, PromptNextSessionAnswerYes(true)),
                K::Esc | K::Char('n') => dsp!(self, timer, PromptNextSessionAnswerYes(false)),
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
                        let pomo = &self.runtime.core().config().pomodoro;
                        dsp!(self, setting, StartEdit(pomo));
                    }
                }
                K::Char('1') => dsp!(self, setting, SectionSelect(0)),
                K::Char('2') => dsp!(self, setting, SectionSelect(1)),
                K::Char('3') => dsp!(self, setting, SectionSelect(2)),
                K::Char('s') => dsp!(self, setting, SaveConfig),
                K::Esc => dsp!(self, runtime, Page::Timer),
                K::Char('q') => self.runtime.dispatch(Msg::Quit),
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
        log::debug!("addr settings {:p}", &self.settings as *const _);
        if let Event::Key(key) = event
            && let Some(prompt) = self.settings.prompt_state_mut()
        {
            prompt.text_state.handle_key_event(key);

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
