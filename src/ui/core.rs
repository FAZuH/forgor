use crate::config::Alarm;
use crate::config::Alarms;
use crate::config::Config;
use crate::config::Hooks;
use crate::config::Timers;
use crate::model::Mode;
use crate::model::Pomodoro;
use crate::repo::model::PomodoroState;
use crate::ui::prelude::*;

// type IPomodoro = Box<dyn Updateable<PomodoroMsg, PomodoroCmd>>;

#[derive(Debug, Clone, PartialEq)]
pub enum Msg {
    // Delegated to sub-models inside AppCore.
    Pomodoro(PomodoroMsg),
    Config(ConfigMsg),
    Router(RouterMsg),

    // System messages.
    /// Emitted by the Runner every 1 second.
    Tick,

    // Views
    ViewTimerCmd(TimerCmd),
    ViewSettingsCmd(SettingsCmd),

    // Effect results (returned by EffectHandler::execute).
    SessionCreated {
        id: i32,
    },
    SessionUpdated,
    SessionEnded,
    SessionsClosed,
    ConfigSaved(ConfigSaveResult),
    NotificationSent(Result<(), String>),

    // Duplicate instance warning
    DuplicateWarningDismiss,
    DuplicateWarningQuit,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Cmd {
    // Sound
    PlaySound(Alarm),
    StopSound,
    // Notification
    SendNotification(Mode),
    // Database
    NewSession {
        task_id: Option<i32>,
        state: PomodoroState,
    },
    UpdateSession {
        id: i32,
    },
    EndSession {
        id: i32,
    },
    CloseAllSessions,
    // Toast
    ShowToast {
        message: String,
        kind: ToastType,
    },
    // Persistence
    SaveConfig(Box<Config>), // Box, because Config is large.
    RunHook(String),
}

pub struct AppCore<E: EffectHandler> {
    pomodoro: Pomodoro,
    config: Config,
    router: Router,
    effects: E,

    active_session_id: Option<i32>,
    config_snapshot: Config,
    is_prompting_transition: bool,
    show_duplicate_warning: bool,
}

impl<E: EffectHandler> AppCore<E> {
    pub fn new(pomodoro: Pomodoro, config: Config, effects: E, is_duplicate: bool) -> Self {
        Self {
            effects,
            pomodoro,
            router: Router::new(Page::Timer),
            config_snapshot: config.clone(),
            config,
            active_session_id: None,
            is_prompting_transition: false,
            show_duplicate_warning: is_duplicate,
        }
    }

    /// Dispatch a message and process all resulting effects
    /// synchronously until the message queue is drained.
    pub fn dispatch(&mut self, msg: Msg) {
        let cmds = self.update(msg);
        for cmd in cmds {
            self.execute_effect(cmd);
        }
    }

    pub fn execute_effect(&mut self, cmd: Cmd) {
        for res in self.effects.execute(cmd) {
            self.dispatch(res);
        }
    }

    // -- Getters --

    pub fn router(&self) -> &Router {
        &self.router
    }

    pub fn router_mut(&mut self) -> &mut Router {
        &mut self.router
    }

    pub fn pomodoro(&self) -> &Pomodoro {
        &self.pomodoro
    }

    pub fn config(&self) -> &Config {
        &self.config
    }

    pub fn effects(&self) -> &E {
        &self.effects
    }

    pub fn effects_mut(&mut self) -> &mut E {
        &mut self.effects
    }

    pub fn is_prompting_transition(&self) -> bool {
        self.is_prompting_transition
    }

    pub fn is_config_dirty(&self) -> bool {
        self.config != self.config_snapshot
    }

    pub fn show_duplicate_warning(&self) -> bool {
        self.show_duplicate_warning
    }
}

impl<E: EffectHandler> Updateable<Msg, Cmd> for AppCore<E> {
    fn update(&mut self, msg: Msg) -> Vec<Cmd> {
        let mut ret = Vec::new();
        match msg {
            Msg::Pomodoro(msg) => {
                for cmd in self.pomodoro.update(msg) {
                    ret.extend(self.translate_pomodoro_cmd(cmd));
                }
            }
            Msg::Config(msg) => {
                for cmd in self.config.update(msg) {
                    ret.extend(self.translate_config_cmd(cmd))
                }
            }
            Msg::Router(msg) => {
                for cmd in self.router.update(msg) {
                    ret.extend(self.translate_router_cmd(cmd));
                }
            }
            Msg::ViewTimerCmd(cmd) => ret.extend(self.translate_timer_cmd(cmd)),
            Msg::ViewSettingsCmd(cmd) => ret.extend(self.translate_settings_cmd(cmd)),
            Msg::Tick => ret.extend(self.handle_tick()),
            Msg::SessionCreated { id } => self.active_session_id = Some(id),
            Msg::SessionUpdated => {}
            Msg::SessionEnded => self.active_session_id = None,
            Msg::SessionsClosed => {}
            Msg::ConfigSaved(result) => ret.extend(self.handle_config_saved(result)),
            Msg::NotificationSent(_) => {}
            Msg::DuplicateWarningDismiss => ret.extend(self.handle_duplicate_warning_dismiss()),
            Msg::DuplicateWarningQuit => ret.extend(self.update(Msg::Router(RouterMsg::Quit))),
        }
        ret
    }
}

impl<E: EffectHandler> AppCore<E> {
    fn handle_tick(&mut self) -> Vec<Cmd> {
        let mut ret = vec![];

        // Bump the session timestamp on every tick.
        if let Some(id) = self.active_session_id {
            ret.push(Cmd::UpdateSession { id });
        }

        ret.extend(self.update(Msg::Pomodoro(PomodoroMsg::Tick)));

        ret
    }

    fn handle_config_saved(&mut self, result: ConfigSaveResult) -> Vec<Cmd> {
        match result {
            ConfigSaveResult::Ok => {
                self.config_snapshot = self.config.clone();
                vec![Cmd::ShowToast {
                    message: "Settings saved!".into(),
                    kind: ToastType::Success,
                }]
            }
            ConfigSaveResult::Err(e) => vec![Cmd::ShowToast {
                message: format!("Failed to save: {e}"),
                kind: ToastType::Error,
            }],
        }
    }

    fn handle_session_end(&mut self, curr_mode: Mode, next_mode: Mode) -> Vec<Cmd> {
        let mut ret = Vec::new();

        // End the active session (.take() so only fires once).
        if let Some(id) = self.active_session_id.take() {
            ret.push(Cmd::EndSession { id });
        }

        // Fire effects: sound, notification, hook.
        // Skipped if already prompting (effects already fired on first SessionEnd).
        let config = &self.config_snapshot;
        let auto_next = self.should_auto_next(&curr_mode, &config.pomodoro.timer);
        let should_fire = auto_next || !self.is_prompting_transition;

        if should_fire {
            let alarm = self.alarm_for(&next_mode, &config.pomodoro.alarm).clone();
            let hook = self.hook_for(&curr_mode, &config.pomodoro.hook).to_string();

            ret.push(Cmd::SendNotification(next_mode));
            ret.push(Cmd::PlaySound(alarm));
            ret.push(Cmd::RunHook(hook));
        }

        if auto_next {
            // Advance to the next mode and start a fresh session.
            ret.extend(self.update(Msg::Pomodoro(PomodoroMsg::NextSession)));
        } else {
            self.is_prompting_transition = true;
        }

        ret
    }

    fn handle_duplicate_warning_dismiss(&mut self) -> Vec<Cmd> {
        let mut ret = vec![];

        self.show_duplicate_warning = false;

        ret.push(Cmd::CloseAllSessions);
        ret.extend(self.update(Msg::Router(RouterMsg::GoTo(Page::Timer))));

        let auto_start = self.config.pomodoro.timer.auto_start_on_launch;
        if auto_start {
            ret.extend(self.update(Msg::Pomodoro(PomodoroMsg::Start)));
        } else {
            ret.extend(self.update(Msg::Pomodoro(PomodoroMsg::StartPaused)));
        }

        ret
    }

    fn should_auto_next(&self, mode: &Mode, timer: &Timers) -> bool {
        match mode {
            Mode::Focus => timer.auto_focus,
            Mode::LongBreak => timer.auto_long,
            Mode::ShortBreak => timer.auto_short,
        }
    }

    fn alarm_for<'l>(&self, mode: &Mode, alarm: &'l Alarms) -> &'l Alarm {
        match mode {
            Mode::Focus => &alarm.focus,
            Mode::LongBreak => &alarm.long,
            Mode::ShortBreak => &alarm.short,
        }
    }

    fn hook_for<'l>(&self, mode: &Mode, hook: &'l Hooks) -> &'l str {
        match mode {
            Mode::Focus => &hook.focus,
            Mode::LongBreak => &hook.long,
            Mode::ShortBreak => &hook.short,
        }
    }
}

impl<E: EffectHandler> AppCore<E> {
    fn translate_pomodoro_cmd(&mut self, cmd: PomodoroCmd) -> Vec<Cmd> {
        let mut ret = Vec::new();
        match cmd {
            PomodoroCmd::Started(mode) => {
                ret.push(Cmd::NewSession {
                    task_id: None,
                    state: mode.into(),
                });
            }
            PomodoroCmd::StartedPaused => {}
            PomodoroCmd::SessionEnd {
                curr_mode,
                next_mode,
            } => {
                ret.extend(self.handle_session_end(curr_mode, next_mode));
            }
            PomodoroCmd::NextSession(mode) => {
                ret.push(Cmd::NewSession {
                    task_id: None,
                    state: mode.into(),
                });
            }
            PomodoroCmd::SessionPaused => {
                if let Some(id) = self.active_session_id.take() {
                    ret.push(Cmd::EndSession { id });
                }
            }
            PomodoroCmd::SessionResumed(mode) => {
                ret.push(Cmd::NewSession {
                    task_id: None,
                    state: mode.into(),
                });
            }
            PomodoroCmd::SessionSkipped(mode) => {
                if let Some(id) = self.active_session_id.take() {
                    ret.push(Cmd::EndSession { id });
                }
                ret.push(Cmd::NewSession {
                    task_id: None,
                    state: mode.into(),
                });
            }
        }
        ret
    }

    fn translate_config_cmd(&mut self, cmd: ConfigCmd) -> Vec<Cmd> {
        let ret = vec![];
        match cmd {
            ConfigCmd::None => {}
        }
        ret
    }

    fn translate_router_cmd(&mut self, cmd: RouterCmd) -> Vec<Cmd> {
        let ret = vec![];
        match cmd {
            RouterCmd::Quit => {
                // if self.is_config_dirty() {
                //     ret.push(Cmd::)
                // } else {
                //     ret.push(Cmd::Quit);
                // }
            }
        }
        ret
    }

    fn translate_timer_cmd(&mut self, cmd: TimerCmd) -> Vec<Cmd> {
        let mut ret = vec![Cmd::StopSound];

        match cmd {
            TimerCmd::PromptTransitionAnsweredYes => {
                ret.extend(self.update(Msg::Pomodoro(PomodoroMsg::NextSession)))
            }
            TimerCmd::PromptTransitionAnsweredNo => {
                ret.extend(self.update(Msg::Pomodoro(PomodoroMsg::NextSession)));
                ret.extend(self.update(Msg::Pomodoro(PomodoroMsg::Pause)))
            }
        };
        self.is_prompting_transition = false;

        ret
    }

    fn translate_settings_cmd(&mut self, cmd: SettingsCmd) -> Vec<Cmd> {
        let mut ret = vec![];
        match cmd {
            SettingsCmd::SaveEdit(msg) => {
                ret.extend(self.update(Msg::Config(msg)));
            }
            SettingsCmd::SaveConfig => {
                ret.push(Cmd::SaveConfig(Box::new(self.config.clone())));
            }
            SettingsCmd::ShowToast { message, r#type } => {
                ret.push(Cmd::ShowToast {
                    message,
                    kind: r#type,
                });
            }
        }
        ret
    }
}

impl From<Mode> for PomodoroState {
    fn from(value: Mode) -> Self {
        match value {
            Mode::Focus => Self::Focus,
            Mode::LongBreak => Self::LongBreak,
            Mode::ShortBreak => Self::ShortBreak,
        }
    }
}

/// Outcome of a config-save effect so AppCore can update its dirty flag.
#[derive(Debug, Clone, PartialEq)]
pub enum ConfigSaveResult {
    Ok,
    Err(String),
}

impl From<Result<(), String>> for ConfigSaveResult {
    fn from(r: Result<(), String>) -> Self {
        match r {
            Ok(()) => ConfigSaveResult::Ok,
            Err(e) => ConfigSaveResult::Err(e),
        }
    }
}
