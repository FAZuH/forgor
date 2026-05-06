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

/// Events and commands dispatched to the core state machine.
#[derive(Debug, Clone, PartialEq)]
pub enum Msg {
    // Delegated to sub-models inside AppCore.
    Pomodoro(PomodoroMsg),
    Config(ConfigMsg),
    Router(RouterMsg),

    // System messages.
    Quit,
    ForceQuit,
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

    // Unsaved changes warning
    UnsavedWarningSave,
    UnsavedWarningCancel,
    UnsavedWarningQuit,

    // Timer reset warning
    ResetWarningShow,
    ResetWarningProceed,
    ResetWarningCancel,
}

/// Side-effects to be executed by the EffectHandler.
#[derive(Debug, Clone, PartialEq)]
pub enum Cmd {
    Quit,
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

/// The single source of truth for the application's business state and configuration.
pub struct AppCore<E: EffectHandler> {
    pomodoro: Pomodoro,
    config: Config,
    router: Router,
    effects: E,

    active_session_id: Option<i32>,
    config_snapshot: Config,
    overlay: Option<Overlay>,
    is_quit: bool,
}

/// Represents the active modal overlay blocking the main interface.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Overlay {
    PromptingTransition,
    DuplicateWarning,
    UnsavedWarning,
    ResetWarning,
}

impl<E: EffectHandler> AppCore<E> {
    pub fn new(pomodoro: Pomodoro, config: Config, effects: E, is_duplicate: bool) -> Self {
        let overlay = if is_duplicate {
            Some(Overlay::DuplicateWarning)
        } else {
            None
        };

        Self {
            effects,
            pomodoro,
            router: Router::new(Page::Timer),
            config_snapshot: config.clone(),
            config,

            active_session_id: None,
            overlay,
            is_quit: false,
        }
    }

    /// Dispatches a message into the state machine and resolves
    /// any resulting side-effects synchronously.
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

    pub fn is_config_dirty(&self) -> bool {
        self.config != self.config_snapshot
    }

    pub fn overlay(&self) -> Option<Overlay> {
        self.overlay
    }

    pub fn set_overlay(&mut self, overlay: Option<Overlay>) {
        self.overlay = overlay;
    }

    pub fn is_quit(&self) -> bool {
        self.is_quit
    }

    pub(crate) fn update_config_snapshot(&mut self) {
        self.config_snapshot = self.config.clone();
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
            Msg::DuplicateWarningDismiss => self.overlay = None,
            Msg::DuplicateWarningQuit => ret.extend(self.update(Msg::Quit)),
            Msg::UnsavedWarningSave => {
                ret.extend(self.update(Msg::ViewSettingsCmd(SettingsCmd::SaveConfig)));
                ret.extend(self.update(Msg::Quit));
                self.overlay = None;
            }
            Msg::UnsavedWarningQuit => ret.extend(self.update(Msg::ForceQuit)),
            Msg::UnsavedWarningCancel => self.overlay = None,
            Msg::Quit => ret.extend(self.handle_quit()),
            Msg::ForceQuit => self.is_quit = true,
            Msg::ResetWarningProceed => {
                ret.extend(self.update(Msg::Pomodoro(PomodoroMsg::ResetSession)));
                self.overlay = None;
            }
            Msg::ResetWarningCancel => self.overlay = None,
            Msg::ResetWarningShow => self.overlay = Some(Overlay::ResetWarning),
        }
        ret
    }
}

impl<E: EffectHandler> AppCore<E> {
    fn handle_quit(&mut self) -> Vec<Cmd> {
        let mut ret = vec![];
        if self.is_config_dirty() {
            self.overlay = Some(Overlay::UnsavedWarning);
        } else {
            self.is_quit = true;
            ret.push(Cmd::Quit);
        }
        ret
    }

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

    pub(crate) fn take_active_session_id(&mut self) -> Option<i32> {
        self.active_session_id.take()
    }

    pub(crate) fn handle_session_end(&mut self, curr_mode: Mode, next_mode: Mode) -> Vec<Cmd> {
        let mut ret = Vec::new();

        // End the active session (.take() so only fires once).
        if let Some(id) = self.active_session_id.take() {
            ret.push(Cmd::EndSession { id });
        }

        // Fire effects: sound, notification, hook.
        // Skipped if already prompting (effects already fired on first SessionEnd).
        let config = &self.config_snapshot;
        let auto_next = self.should_auto_next(&curr_mode, &config.pomodoro.timer);
        let should_fire = auto_next || self.overlay != Some(Overlay::PromptingTransition);

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
            self.overlay = Some(Overlay::PromptingTransition);
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

impl From<Mode> for PomodoroState {
    fn from(value: Mode) -> Self {
        match value {
            Mode::Focus => Self::Focus,
            Mode::LongBreak => Self::LongBreak,
            Mode::ShortBreak => Self::ShortBreak,
        }
    }
}

/// Outcome of a config-save effect so that AppCore can update its dirty flag.
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

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use std::time::Duration;

    use super::*;

    struct MockEffects;
    impl EffectHandler for MockEffects {
        fn execute(&mut self, _cmd: Cmd) -> Vec<Msg> {
            vec![]
        }
    }

    fn test_config() -> Config {
        Config::new(PathBuf::from("/tmp/test_tomo_config"))
    }

    fn test_pomodoro() -> Pomodoro {
        Pomodoro::default()
    }

    #[test]
    fn duplicate_on_launch() {
        let core = AppCore::new(test_pomodoro(), test_config(), MockEffects, true);
        assert_eq!(core.overlay(), Some(Overlay::DuplicateWarning));
    }

    #[test]
    fn no_duplicate_no_overlay() {
        let core = AppCore::new(test_pomodoro(), test_config(), MockEffects, false);
        assert_eq!(core.overlay(), None);
    }

    #[test]
    fn tick_updates_active_session() {
        let mut core = AppCore::new(test_pomodoro(), test_config(), MockEffects, false);
        core.update(Msg::SessionCreated { id: 42 });

        let cmds = core.update(Msg::Tick);

        assert!(
            cmds.iter()
                .any(|c| matches!(c, Cmd::UpdateSession { id: 42 }))
        );
    }

    #[test]
    fn tick_without_session() {
        let mut core = AppCore::new(test_pomodoro(), test_config(), MockEffects, false);
        let cmds = core.update(Msg::Tick);

        assert!(!cmds.iter().any(|c| matches!(c, Cmd::UpdateSession { .. })));
    }

    #[test]
    fn quit_with_dirty_config_shows_warning() {
        let mut core = AppCore::new(test_pomodoro(), test_config(), MockEffects, false);
        let _ = core.update(Msg::Config(ConfigMsg::TimerAutoFocus));

        let cmds = core.update(Msg::Quit);

        assert_eq!(core.overlay(), Some(Overlay::UnsavedWarning));
        assert!(!core.is_quit());
        assert!(!cmds.iter().any(|c| matches!(c, Cmd::Quit)));
    }

    #[test]
    fn quit_with_clean_config_exits() {
        let mut core = AppCore::new(test_pomodoro(), test_config(), MockEffects, false);
        let cmds = core.update(Msg::Quit);

        assert!(core.is_quit());
        assert!(cmds.iter().any(|c| matches!(c, Cmd::Quit)));
        assert_eq!(core.overlay(), None);
    }

    #[test]
    fn overlay_dismiss_messages() {
        let mut core = AppCore::new(test_pomodoro(), test_config(), MockEffects, false);

        core.set_overlay(Some(Overlay::DuplicateWarning));
        core.update(Msg::DuplicateWarningDismiss);
        assert_eq!(core.overlay(), None);

        core.set_overlay(Some(Overlay::UnsavedWarning));
        core.update(Msg::UnsavedWarningCancel);
        assert_eq!(core.overlay(), None);

        core.set_overlay(Some(Overlay::ResetWarning));
        core.update(Msg::ResetWarningCancel);
        assert_eq!(core.overlay(), None);
    }

    #[test]
    fn reset_warning_proceed() {
        let mut core = AppCore::new(test_pomodoro(), test_config(), MockEffects, false);
        core.set_overlay(Some(Overlay::ResetWarning));

        let _cmds = core.update(Msg::ResetWarningProceed);

        assert_eq!(core.overlay(), None);
    }

    #[test]
    fn reset_warning_show() {
        let mut core = AppCore::new(test_pomodoro(), test_config(), MockEffects, false);
        assert_eq!(core.overlay(), None);

        core.update(Msg::ResetWarningShow);
        assert_eq!(core.overlay(), Some(Overlay::ResetWarning));
    }

    #[test]
    fn unsaved_warning_save_and_quit() {
        let mut core = AppCore::new(test_pomodoro(), test_config(), MockEffects, false);
        core.set_overlay(Some(Overlay::UnsavedWarning));

        let cmds = core.update(Msg::UnsavedWarningSave);

        assert_eq!(core.overlay(), None);
        assert!(cmds.iter().any(|c| matches!(c, Cmd::SaveConfig(_))));
        assert!(cmds.iter().any(|c| matches!(c, Cmd::Quit)));
    }

    #[test]
    fn unsaved_warning_quit() {
        let mut core = AppCore::new(test_pomodoro(), test_config(), MockEffects, false);
        core.set_overlay(Some(Overlay::UnsavedWarning));

        core.update(Msg::UnsavedWarningQuit);
        assert!(core.is_quit());
    }

    #[test]
    fn duplicate_warning_quit() {
        let mut core = AppCore::new(test_pomodoro(), test_config(), MockEffects, false);
        core.set_overlay(Some(Overlay::DuplicateWarning));

        let cmds = core.update(Msg::DuplicateWarningQuit);
        assert!(cmds.iter().any(|c| matches!(c, Cmd::Quit)));
    }

    #[test]
    fn config_saved_ok() {
        let mut core = AppCore::new(test_pomodoro(), test_config(), MockEffects, false);
        let _ = core.update(Msg::Config(ConfigMsg::TimerAutoFocus));
        assert!(core.is_config_dirty());

        let cmds = core.update(Msg::ConfigSaved(ConfigSaveResult::Ok));

        assert!(!core.is_config_dirty());
        assert!(cmds.iter().any(|c| matches!(
            c,
            Cmd::ShowToast {
                kind: ToastType::Success,
                ..
            }
        )));
    }

    #[test]
    fn config_saved_err() {
        let mut core = AppCore::new(test_pomodoro(), test_config(), MockEffects, false);
        let _ = core.update(Msg::Config(ConfigMsg::TimerAutoFocus));
        assert!(core.is_config_dirty());

        let cmds = core.update(Msg::ConfigSaved(ConfigSaveResult::Err("disk full".into())));

        assert!(core.is_config_dirty());
        assert!(cmds.iter().any(|c| matches!(
            c,
            Cmd::ShowToast {
                kind: ToastType::Error,
                ..
            }
        )));
    }

    #[test]
    fn session_end_emits_effects_and_prompt() {
        let pomo = Pomodoro::new(
            Duration::from_secs(0),
            Duration::from_mins(10),
            Duration::from_mins(5),
            4,
        );
        let mut core = AppCore::new(pomo, test_config(), MockEffects, false);

        core.update(Msg::Pomodoro(PomodoroMsg::Start));
        core.update(Msg::SessionCreated { id: 1 });

        let cmds = core.update(Msg::Pomodoro(PomodoroMsg::Tick));

        assert!(cmds.iter().any(|c| matches!(c, Cmd::SendNotification(_))));
        assert!(cmds.iter().any(|c| matches!(c, Cmd::PlaySound(_))));
        assert!(cmds.iter().any(|c| matches!(c, Cmd::RunHook(_))));
        assert!(cmds.iter().any(|c| matches!(c, Cmd::EndSession { id: 1 })));
        assert_eq!(core.overlay(), Some(Overlay::PromptingTransition));
    }

    #[test]
    fn session_end_auto_next() {
        let pomo = Pomodoro::new(
            Duration::from_secs(0),
            Duration::from_mins(10),
            Duration::from_mins(5),
            4,
        );
        let mut config = test_config();
        config.pomodoro.timer.auto_focus = true;

        let mut core = AppCore::new(pomo, config, MockEffects, false);
        core.update(Msg::Pomodoro(PomodoroMsg::Start));
        core.update(Msg::SessionCreated { id: 1 });

        let cmds = core.update(Msg::Pomodoro(PomodoroMsg::Tick));

        assert!(cmds.iter().any(|c| matches!(c, Cmd::SendNotification(_))));
        assert!(cmds.iter().any(|c| matches!(c, Cmd::PlaySound(_))));
        assert!(cmds.iter().any(|c| matches!(c, Cmd::RunHook(_))));
        assert_eq!(core.overlay(), None);
        assert!(cmds.iter().any(|c| matches!(c, Cmd::NewSession { .. })));
    }

    #[test]
    fn session_end_idempotent_effects() {
        let pomo = Pomodoro::new(
            Duration::from_secs(0),
            Duration::from_mins(10),
            Duration::from_mins(5),
            4,
        );
        let mut core = AppCore::new(pomo, test_config(), MockEffects, false);

        core.update(Msg::Pomodoro(PomodoroMsg::Start));
        core.update(Msg::SessionCreated { id: 1 });

        // First session end fires effects
        core.update(Msg::Pomodoro(PomodoroMsg::Tick));
        assert_eq!(core.overlay(), Some(Overlay::PromptingTransition));

        // Another tick while prompting should not fire duplicate effects
        core.update(Msg::SessionCreated { id: 2 });
        let cmds = core.update(Msg::Pomodoro(PomodoroMsg::Tick));
        assert!(!cmds.iter().any(|c| matches!(c, Cmd::SendNotification(_))));
        assert!(!cmds.iter().any(|c| matches!(c, Cmd::PlaySound(_))));
        assert!(!cmds.iter().any(|c| matches!(c, Cmd::RunHook(_))));
    }

    #[test]
    fn from_mode_to_pomodoro_state() {
        assert_eq!(PomodoroState::from(Mode::Focus), PomodoroState::Focus);
        assert_eq!(
            PomodoroState::from(Mode::LongBreak),
            PomodoroState::LongBreak
        );
        assert_eq!(
            PomodoroState::from(Mode::ShortBreak),
            PomodoroState::ShortBreak
        );
    }

    #[test]
    fn config_save_result_from_ok() {
        let r: ConfigSaveResult = Ok::<(), String>(()).into();
        assert_eq!(r, ConfigSaveResult::Ok);
    }

    #[test]
    fn config_save_result_from_err() {
        let r: ConfigSaveResult = Result::<(), String>::Err("oops".into()).into();
        assert_eq!(r, ConfigSaveResult::Err("oops".into()));
    }
}
