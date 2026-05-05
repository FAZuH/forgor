use crate::config::Alarm;
use crate::config::Config;
use crate::model::Mode;
use crate::model::Pomodoro;
use crate::repo::model::PomodoroState;
use crate::ui::prelude::*;

#[derive(Debug, Clone, PartialEq)]
pub enum Msg {
    // Delegated to sub-models inside AppCore.
    Pomodoro(PomodoroMsg),
    Config(ConfigMsg),

    // System messages.
    /// Emitted by the Runner every 1 second.
    Tick,
    /// Emitted by the Runner when the user requests quit.
    Quit,

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
    SaveConfig(Box<Config>),
    RunHook(String),
    // Views
}

pub struct AppCore {
    pomodoro: Pomodoro,
    config: Config,
    router: Router,

    active_session_id: Option<i32>,
    config_snapshot: Option<Config>,
    is_prompting_transition: bool,
}

impl AppCore {
    pub fn new(pomodoro: Pomodoro, config: Config) -> Self {
        Self {
            pomodoro,
            router: Router::new(Page::Timer),
            config_snapshot: Some(config.clone()),
            config,
            active_session_id: None,
            is_prompting_transition: false,
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

    pub fn is_prompting_transition(&self) -> bool {
        self.is_prompting_transition
    }

    pub fn is_config_dirty(&self) -> bool {
        match &self.config_snapshot {
            Some(snap) => self.config != *snap,
            None => true,
        }
    }

    pub fn update(&mut self, msg: Msg) -> Vec<Cmd> {
        let mut ret = Vec::new();
        match msg {
            Msg::Pomodoro(pm) => {
                let cmds = self.pomodoro.update(pm);
                ret.extend(self.translate_pomodoro_cmds(cmds));
            }
            Msg::Config(cm) => {
                self.config.update(cm);
            }
            Msg::ViewTimerCmd(cmd) => {
                ret.extend(self.translate_timer_cmds(vec![cmd]));
            }
            Msg::ViewSettingsCmd(cmd) => {
                ret.extend(self.translate_settings_cmds(vec![cmd]));
            }
            Msg::Tick => ret.extend(self.handle_tick()),
            Msg::Quit => self.router.navigate(Navigation::Quit),
            Msg::SessionCreated { id } => self.active_session_id = Some(id),
            Msg::SessionUpdated => {}
            Msg::SessionEnded => self.active_session_id = None,
            Msg::SessionsClosed => {}
            Msg::ConfigSaved(result) => ret.extend(self.handle_config_saved(result)),
            Msg::NotificationSent(_) => {}
        }
        ret
    }
}

impl AppCore {
    fn handle_tick(&mut self) -> Vec<Cmd> {
        let mut ret = vec![];

        // Bump the session timestamp on every tick.
        if let Some(id) = self.active_session_id {
            ret.push(Cmd::UpdateSession { id });
        }

        let pomo_cmds = self.pomodoro.update(PomodoroMsg::Tick);
        ret.extend(self.translate_pomodoro_cmds(pomo_cmds));

        ret
    }

    fn handle_config_saved(&mut self, result: ConfigSaveResult) -> Vec<Cmd> {
        match result {
            ConfigSaveResult::Ok => {
                self.config_snapshot = Some(self.config.clone());
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

    fn should_auto_next(&self) -> bool {
        let timer = &self.config.pomodoro.timer;
        match self.pomodoro.mode() {
            Mode::Focus => timer.auto_focus,
            Mode::LongBreak => timer.auto_long,
            Mode::ShortBreak => timer.auto_short,
        }
    }

    fn alarm_for(&self, mode: Mode) -> &Alarm {
        let alarms = &self.config.pomodoro.alarm;
        match mode {
            Mode::Focus => &alarms.focus,
            Mode::LongBreak => &alarms.long,
            Mode::ShortBreak => &alarms.short,
        }
    }

    fn hook_for(&self, mode: Mode) -> &str {
        let hooks = &self.config.pomodoro.hook;
        match mode {
            Mode::Focus => &hooks.focus,
            Mode::LongBreak => &hooks.long,
            Mode::ShortBreak => &hooks.short,
        }
    }
}

impl AppCore {
    fn translate_pomodoro_cmds(&mut self, cmds: Vec<PomodoroCmd>) -> Vec<Cmd> {
        let mut ret = Vec::new();
        for cmd in cmds {
            match cmd {
                PomodoroCmd::Started => {
                    ret.push(Cmd::NewSession {
                        task_id: None,
                        state: self.pomodoro.mode().into(),
                    });
                }
                PomodoroCmd::StartedPaused => {}
                PomodoroCmd::SessionEnd => {
                    ret.extend(self.handle_session_end());
                }
                PomodoroCmd::NextSession => {
                    ret.push(Cmd::NewSession {
                        task_id: None,
                        state: self.pomodoro.mode().into(),
                    });
                }
                PomodoroCmd::SessionPaused => {
                    if let Some(id) = self.active_session_id.take() {
                        ret.push(Cmd::EndSession { id });
                    }
                }
                PomodoroCmd::SessionResumed => {
                    ret.push(Cmd::NewSession {
                        task_id: None,
                        state: self.pomodoro.mode().into(),
                    });
                }
                PomodoroCmd::SessionSkipped => {
                    if let Some(id) = self.active_session_id.take() {
                        ret.push(Cmd::EndSession { id });
                    }
                    ret.push(Cmd::NewSession {
                        task_id: None,
                        state: self.pomodoro.mode().into(),
                    });
                }
            }
        }
        ret
    }

    fn translate_timer_cmds(&mut self, cmds: Vec<TimerCmd>) -> Vec<Cmd> {
        let mut ret = vec![];
        for cmd in cmds {
            ret.push(Cmd::StopSound);
            let pomo = match cmd {
                TimerCmd::PromptTransitionAnsweredYes => {
                    self.pomodoro.update(PomodoroMsg::NextSession)
                }
                TimerCmd::PromptTransitionAnsweredNo => self.pomodoro.update(PomodoroMsg::Pause),
            };
            ret.extend(self.translate_pomodoro_cmds(pomo));
        }
        ret
    }

    fn translate_settings_cmds(&mut self, cmds: Vec<SettingsCmd>) -> Vec<Cmd> {
        let mut output = vec![];
        for cmd in cmds {
            match cmd {
                SettingsCmd::SaveEdit(config_msg) => {
                    self.config.update(config_msg);
                }
                SettingsCmd::SaveConfig => {
                    output.push(Cmd::SaveConfig(Box::new(self.config.clone())));
                }
                SettingsCmd::ShowToast { message, r#type } => {
                    output.push(Cmd::ShowToast {
                        message,
                        kind: r#type,
                    });
                }
            }
        }
        output
    }

    fn handle_session_end(&mut self) -> Vec<Cmd> {
        let auto_next = self.should_auto_next();
        let current_mode = self.pomodoro.mode();
        let next_mode = self.pomodoro.next_mode();
        let mut ret = Vec::new();

        // End the active session (only fires once — id is taken).
        if let Some(id) = self.active_session_id.take() {
            ret.push(Cmd::EndSession { id });
        }

        // Fire effects: sound, notification, hook.
        // Skipped if already prompting (effects already fired on first SessionEnd).
        let should_fire = auto_next || !self.is_prompting_transition;
        if should_fire {
            ret.push(Cmd::PlaySound(self.alarm_for(next_mode).clone()));
            ret.push(Cmd::SendNotification(next_mode));
            ret.push(Cmd::RunHook(self.hook_for(current_mode).to_string()));
        }

        if auto_next {
            // Advance to the next mode and start a fresh session.
            let next = self.pomodoro.update(PomodoroMsg::NextSession);
            ret.extend(self.translate_pomodoro_cmds(next));
        } else {
            self.is_prompting_transition = true;
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
