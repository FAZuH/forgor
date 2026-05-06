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
use crate::ui::core::Overlay;
use crate::ui::prelude::*;
use crate::ui::tui::TuiEffectHandler;
use crate::ui::tui::TuiError;
use crate::ui::tui::backend::Tui;
use crate::ui::tui::view::*;
use crate::ui::update::PomodoroMsg;

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
            self.dispatch_pomo(PomodoroMsg::Start);
        } else {
            self.dispatch_pomo(PomodoroMsg::StartPaused);
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
                        self.timer
                            .render(f, self.core.pomodoro(), is_prompting_transition)
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
            self.dispatch_core(Msg::Tick);
        }
    }

    // ---------------------------------------------------------
    //  ___ ___ ___ ___  _ _____ ___ _  _
    // |   \_ _/ __| _ \/_\_   _/ __| || |
    // | |) | |\__ \  _/ _ \| || (__| __ |
    // |___/___|___/_|/_/ \_\_| \___|_||_|
    // ---------------------------------------------------------

    fn dispatch_core(&mut self, msg: Msg) {
        log::trace!("msg core: {:?}", msg);
        self.core.dispatch(msg);
        self.redraw();
    }

    fn dispatch_pomo(&mut self, msg: PomodoroMsg) {
        log::trace!("msg pomo: {:?}", msg);
        self.core.dispatch(Msg::Pomodoro(msg));
        self.redraw();
    }

    fn dispatch_router(&mut self, msg: RouterMsg) {
        log::trace!("msg router: {:?}", msg);
        self.core.dispatch(Msg::Router(msg));
        self.redraw();
    }

    fn dispatch_timer(&mut self, msg: TimerMsg) {
        log::trace!("msg timer: {:?}", msg);
        let cmds = self.timer.update(msg);
        for cmd in cmds {
            self.core.dispatch(Msg::ViewTimerCmd(cmd));
        }
        self.redraw();
    }

    fn dispatch_setting(&mut self, msg: SettingsMsg) {
        let cmds = self.settings.update(msg);
        for cmd in cmds {
            self.core.dispatch(Msg::ViewSettingsCmd(cmd));
        }
        self.redraw();
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
                KeyCode::Char('m') => self.core.execute_effect(Cmd::StopSound),
                KeyCode::Char('q') => self.dispatch_core(Msg::Quit),
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
                K::Enter | K::Char('y') => {
                    self.dispatch_core(Msg::ResetWarningProceed);
                }
                K::Esc | K::Char('n') => {
                    self.dispatch_core(Msg::ResetWarningCancel);
                }
                _ => {}
            }
        }
    }

    fn handle_unsaved_warning(&mut self, event: &Event) {
        if let Event::Key(key) = event {
            use KeyCode as K;
            match key.code {
                K::Enter | K::Char('s') => {
                    self.dispatch_core(Msg::UnsavedWarningSave);
                }
                K::Esc => {
                    self.dispatch_core(Msg::UnsavedWarningCancel);
                }
                K::Char('q') => {
                    self.dispatch_core(Msg::UnsavedWarningQuit);
                }
                _ => {}
            }
        }
    }

    fn handle_duplicate_warning(&mut self, event: &Event) {
        if let Event::Key(key) = event {
            use KeyCode as K;
            match key.code {
                K::Enter | K::Char('y') => {
                    self.dispatch_core(Msg::DuplicateWarningDismiss);
                }
                K::Char('q') | K::Esc | K::Char('n') => {
                    self.dispatch_core(Msg::DuplicateWarningQuit);
                }
                _ => {}
            }
        }
    }

    fn handle_timer(&mut self, event: Event) {
        if self.core.overlay() == Some(Overlay::PromptingTransition) {
            return self.handle_timer_transition(event);
        }

        use KeyCode as K;
        use PomodoroMsg::*;
        if let Event::Key(key) = event {
            match key.code {
                K::Right | K::Char('l') => self.dispatch_pomo(Subtract(Duration::from_secs(30))),
                K::Down | K::Char('j') => self.dispatch_pomo(Subtract(Duration::from_secs(60))),
                K::Left | K::Char('h') => self.dispatch_pomo(Add(Duration::from_secs(30))),
                K::Up | K::Char('k') => self.dispatch_pomo(Add(Duration::from_secs(60))),
                K::Char(' ') => self.dispatch_pomo(TogglePause),
                K::Enter => self.dispatch_pomo(SkipSession),
                K::Backspace => self.dispatch_core(Msg::ResetWarningShow),
                K::Char('s') => self.dispatch_router(RouterMsg::GoTo(Page::Settings)),
                K::Char('/') | K::Char('?') => self.dispatch_timer(TimerMsg::ToggleShowKeybinds),
                _ => {}
            }
        }
    }

    fn handle_timer_transition(&mut self, event: Event) {
        use KeyCode as K;
        if let Event::Key(key) = event {
            match key.code {
                K::Enter | K::Char('y') => {
                    self.dispatch_core(Msg::ViewTimerCmd(TimerCmd::PromptTransitionAnsweredYes))
                }
                K::Esc | K::Char('n') => {
                    self.dispatch_core(Msg::ViewTimerCmd(TimerCmd::PromptTransitionAnsweredNo))
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
                K::Up | K::Char('k') => self.dispatch_setting(SelectUp),
                K::BackTab => self.dispatch_setting(SectionPrev),
                K::Tab => self.dispatch_setting(SectionNext),
                K::Down | K::Char('j') => self.dispatch_setting(SelectDown),
                K::Enter | K::Char(' ') => {
                    let sel = self.settings.selected();
                    if sel.is_toggle() {
                        self.dispatch_setting(ApplyEdit(sel.into()));
                    } else {
                        let pomo = self.core.config().pomodoro.clone();
                        self.dispatch_setting(StartEdit(&pomo));
                    }
                }
                K::Char('1') => self.dispatch_setting(SectionSelect(0)),
                K::Char('2') => self.dispatch_setting(SectionSelect(1)),
                K::Char('3') => self.dispatch_setting(SectionSelect(2)),
                K::Char('s') => self.dispatch_setting(SaveConfig),
                K::Char('c') | K::Char('y') => self.dispatch_setting(SelectForCopy),
                K::Char('v') | K::Char('p') => {
                    let pomo = self.core.config().pomodoro.clone();
                    self.dispatch_setting(CopyValue(&pomo))
                }
                K::Esc => self.dispatch_router(RouterMsg::GoTo(Page::Timer)),
                K::Char('/') | K::Char('?') => self.dispatch_setting(ToggleShowKeybinds),
                _ => {}
            },
            Event::Mouse(mouse) => match mouse.kind {
                MouseEventKind::ScrollDown => self.dispatch_setting(ScrollDown),
                MouseEventKind::ScrollUp => self.dispatch_setting(ScrollUp),
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
                    self.dispatch_setting(SettingsMsg::SaveEdit);
                    self.dispatch_setting(SettingsMsg::CancelEditing);
                }
                Status::Aborted => self.dispatch_setting(SettingsMsg::CancelEditing),
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
