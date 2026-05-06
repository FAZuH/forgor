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
use crate::ui::tui::TuiEffectHandler;
use crate::ui::tui::TuiError;
use crate::ui::tui::backend::Tui;
use crate::ui::tui::view::*;
use crate::ui::update::PomodoroMsg;

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
    ($self:ident, core, $page:expr) => {{
        $self.core.update($page);
        $self.redraw();
    }};
    ($self:ident, pomo, $msg:expr) => {{
        log::trace!("msg config: {:?}", $msg);
        $self.core.dispatch(Msg::Pomodoro($msg));
        $self.redraw();
    }};
    ($self:ident, config, $msg:expr) => {{
        log::trace!("msg config: {:?}", $msg);
        $self.core.dispatch(Msg::Config($msg));
        $self.redraw();
    }};
    ($self:ident, router, $page:expr) => {{
        $self.core.dispatch(Msg::Router($page));
        $self.redraw();
    }};
    ($self:ident, timer, $msg:expr) => {{
        log::trace!("msg timer: {:?}", $msg);
        let cmds = $self.timer.update($msg);
        for cmd in cmds {
            $self.core.dispatch(Msg::ViewTimerCmd(cmd));
        }
        $self.redraw();
    }};
    ($self:ident, msg, $msg:expr) => {{
        $self.core.dispatch($msg);
        $self.redraw();
    }};
    ($self:ident, setting, $msg:expr) => {{
        let cmds = $self.settings.update($msg);
        for cmd in cmds {
            $self.core.dispatch(Msg::ViewSettingsCmd(cmd));
        }
        $self.redraw();
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
        while !self.core.router().is_quit() {
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
                match self.core.router().active_page() {
                    Some(Page::Timer) => {
                        let is_prompting_transition = self.core.is_prompting_transition();
                        self.timer
                            .render(f, self.core.pomodoro(), is_prompting_transition)
                    }
                    Some(Page::Settings) => {
                        let is_config_dirty = self.core.is_config_dirty();
                        self.settings.render(f, self.core.config(), is_config_dirty)
                    }
                    None => {}
                }

                let area = f.area();

                let toast = self.core.effects_mut().toast_mut();
                toast.set_area(area);
                f.render_widget(&**toast, area);

                if self.core.show_duplicate_warning() {
                    f.render_widget(DuplicateWarning::new(), area);
                }
            })?;
        }
        Ok(())
    }

    fn tick(&mut self) {
        if self.should_tick() {
            self.core.effects_mut().toast_mut().tick();
            self.core.dispatch(Msg::Tick);
            self.redraw();
        }
    }

    fn should_tick(&mut self) -> bool {
        self.tick.new_tick() && !self.core.show_duplicate_warning()
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

                if self.core.show_duplicate_warning() {
                    self.handle_duplicate_warning(&event);
                    continue;
                }

                match self.core.router().active_page() {
                    Some(Page::Settings) => self.handle_settings(event),
                    Some(Page::Timer) => self.handle_timer(event),
                    None => dsp!(self, router, RouterMsg::Quit),
                }
            }
        }
        Ok(())
    }

    fn common_handler(&mut self, event: &Event) {
        match event {
            Event::Key(key) if key.code == KeyCode::Char('m') => {
                self.core.execute_effect(Cmd::StopSound)
            }
            Event::Resize(..) => self.redraw(),
            _ => {}
        }
    }

    fn handle_duplicate_warning(&mut self, event: &Event) {
        if let Event::Key(key) = event {
            use KeyCode as K;
            match key.code {
                K::Enter | K::Char('y') => {
                    self.core.dispatch(Msg::DuplicateWarningDismiss);
                    self.redraw();
                }
                K::Char('q') | K::Esc | K::Char('n') => {
                    self.core.dispatch(Msg::DuplicateWarningQuit);
                }
                _ => {}
            }
        }
    }

    fn handle_timer(&mut self, event: Event) {
        if self.core.is_prompting_transition() {
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
                K::Char('q') => dsp!(self, router, RouterMsg::Quit),
                K::Char('s') => dsp!(self, router, RouterMsg::GoTo(Page::Settings)),
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
                        msg,
                        Msg::ViewTimerCmd(TimerCmd::PromptTransitionAnsweredYes)
                    )
                }
                K::Esc | K::Char('n') => {
                    dsp!(
                        self,
                        msg,
                        Msg::ViewTimerCmd(TimerCmd::PromptTransitionAnsweredNo)
                    )
                }
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
                        let pomo = &self.core.config().pomodoro;
                        dsp!(self, setting, StartEdit(pomo));
                    }
                }
                K::Char('1') => dsp!(self, setting, SectionSelect(0)),
                K::Char('2') => dsp!(self, setting, SectionSelect(1)),
                K::Char('3') => dsp!(self, setting, SectionSelect(2)),
                K::Char('s') => dsp!(self, setting, SaveConfig),
                K::Char('c') | K::Char('y') => dsp!(self, setting, SelectForCopy),
                K::Char('v') | K::Char('p') => {
                    dsp!(self, setting, CopyValue(&self.core.config().pomodoro))
                }
                K::Esc => dsp!(self, router, RouterMsg::GoTo(Page::Timer)),
                K::Char('q') => dsp!(self, router, RouterMsg::Quit),
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
        if let Event::Key(key) = event
            && let Some(prompt) = self.settings.prompt_state_mut()
        {
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
