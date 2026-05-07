use crate::config::Alarm;
use crate::config::Alarms;
use crate::config::Config;
use crate::config::Hooks;
use crate::config::Timers;
use crate::model::Mode;
use crate::model::Pomodoro;
use crate::model::Session;
use crate::model::Task;
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
    /// Try to gracefully quit
    Quit,
    /// Force quit without quit handling
    ForceQuit,
    /// Emitted by the Runner every 1 second.
    Tick,

    // View intents promoted to core
    ViewTimerCmd(TimerCmd),
    ViewSettingsCmd(SettingsCmd),

    // Tasks
    Task(TaskMsg),

    // Effect results for further core handling
    TaskResult(TaskResultMsg),
    SessionResult(SessionResultMsg),

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
pub enum Effect {
    Quit,
    // Sound
    PlaySound(Alarm),
    StopSound,
    // Notification
    SendNotification(Mode),
    // Sessions
    Session(SessionEffect),
    // Task
    Task(TaskEffect),
    // Toast
    ShowToast { message: String, kind: ToastType },
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

    current_task: Option<Task>,
    active_session: Option<Session>,
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
    pub fn new(
        pomodoro: Pomodoro,
        config: Config,
        effects: E,
        is_duplicate: bool,
        initial_task: Option<Task>,
    ) -> Self {
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

            current_task: initial_task,
            active_session: None,
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

    pub fn execute_effect(&mut self, cmd: Effect) {
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

    pub fn update_config_snapshot(&mut self) {
        self.config_snapshot = self.config.clone();
    }
}

impl<E: EffectHandler> Updateable<Msg, Effect> for AppCore<E> {
    fn update(&mut self, msg: Msg) -> Vec<Effect> {
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

            Msg::SessionResult(msg) => {
                use SessionResultMsg::*;
                match msg {
                    Added(session) => self.active_session = Some(session),
                    Ended => self.active_session = None,
                }
            }
            Msg::TaskResult(msg) => match msg {
                TaskResultMsg::Added(task) => {
                    self.current_task = Some(task);
                    ret.push(Effect::ShowToast {
                        message: "Task created".into(),
                        kind: ToastType::Success,
                    });
                }
            },
            Msg::Task(msg) => match msg {
                TaskMsg::Add { name, description } => {
                    ret.push(Effect::Task(TaskEffect::Add { name, description }));
                }
                TaskMsg::Select(task) => {
                    self.current_task = Some(task);
                }
                TaskMsg::ClearSelection => {
                    self.current_task = None;
                }
            },
        }
        ret
    }
}

impl<E: EffectHandler> AppCore<E> {
    fn handle_quit(&mut self) -> Vec<Effect> {
        let mut ret = vec![];
        if self.is_config_dirty() {
            self.overlay = Some(Overlay::UnsavedWarning);
        } else {
            self.is_quit = true;
            ret.push(Effect::Quit);
        }
        ret
    }

    fn handle_tick(&mut self) -> Vec<Effect> {
        let mut ret = vec![];

        // Bump the session timestamp on every tick.
        if let Some(ses) = &self.active_session {
            ret.push(Effect::Session(SessionEffect::Update { id: ses.id }));
        }

        ret.extend(self.update(Msg::Pomodoro(PomodoroMsg::Tick)));

        ret
    }

    fn handle_config_saved(&mut self, result: ConfigSaveResult) -> Vec<Effect> {
        match result {
            ConfigSaveResult::Ok => {
                self.config_snapshot = self.config.clone();
                vec![Effect::ShowToast {
                    message: "Settings saved!".into(),
                    kind: ToastType::Success,
                }]
            }
            ConfigSaveResult::Err(e) => vec![Effect::ShowToast {
                message: format!("Failed to save: {e}"),
                kind: ToastType::Error,
            }],
        }
    }

    fn take_active_session_id(&mut self) -> Option<Session> {
        self.active_session.take()
    }

    fn handle_session_end(&mut self, curr_mode: Mode, next_mode: Mode) -> Vec<Effect> {
        let mut ret = Vec::new();

        // End the active session (.take() so only fires once).
        if let Some(ses) = self.active_session.take() {
            ret.push(Effect::Session(SessionEffect::End { id: ses.id }));
        }

        // Fire effects: sound, notification, hook.
        // Skipped if already prompting (effects already fired on first SessionEnd).
        let config = &self.config_snapshot;
        let auto_next = self.should_auto_next(&curr_mode, &config.pomodoro.timer);
        let should_fire = auto_next || self.overlay != Some(Overlay::PromptingTransition);

        if should_fire {
            let alarm = self.alarm_for(&next_mode, &config.pomodoro.alarm).clone();
            let hook = self.hook_for(&curr_mode, &config.pomodoro.hook).to_string();

            ret.push(Effect::SendNotification(next_mode));
            ret.push(Effect::PlaySound(alarm));
            ret.push(Effect::RunHook(hook));
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

impl<E: EffectHandler> AppCore<E> {
    fn translate_pomodoro_cmd(&mut self, cmd: PomodoroCmd) -> Vec<Effect> {
        let mut ret = Vec::new();
        match cmd {
            PomodoroCmd::Started(mode) => {
                ret.push(Effect::Session(SessionEffect::Add {
                    task_id: self.current_task.as_ref().map(|t| t.id),
                    mode,
                }));
            }
            PomodoroCmd::StartedPaused => {}
            PomodoroCmd::SessionEnd {
                curr_mode,
                next_mode,
            } => {
                ret.extend(self.handle_session_end(curr_mode, next_mode));
            }
            PomodoroCmd::NextSession(mode) => {
                ret.push(Effect::Session(SessionEffect::Add {
                    task_id: self.current_task.as_ref().map(|t| t.id),
                    mode,
                }));
            }
            PomodoroCmd::SessionPaused => {
                if let Some(ses) = self.take_active_session_id() {
                    ret.push(Effect::Session(SessionEffect::End { id: ses.id }));
                }
            }
            PomodoroCmd::SessionResumed(mode) => {
                ret.push(Effect::Session(SessionEffect::Add {
                    task_id: self.current_task.as_ref().map(|t| t.id),
                    mode,
                }));
            }
            PomodoroCmd::SessionSkipped(mode) => {
                if let Some(ses) = self.take_active_session_id() {
                    ret.push(Effect::Session(SessionEffect::End { id: ses.id }));
                }
                ret.push(Effect::Session(SessionEffect::Add {
                    task_id: self.current_task.as_ref().map(|t| t.id),
                    mode,
                }));
            }
        }
        ret
    }

    fn translate_config_cmd(&mut self, cmd: ConfigCmd) -> Vec<Effect> {
        let ret = vec![];
        match cmd {
            ConfigCmd::None => {}
        }
        ret
    }

    fn translate_router_cmd(&mut self, cmd: RouterCmd) -> Vec<Effect> {
        let ret = vec![];
        match cmd {
            RouterCmd::None => {}
        }
        ret
    }

    fn translate_timer_cmd(&mut self, cmd: TimerCmd) -> Vec<Effect> {
        let mut ret = vec![Effect::StopSound];

        match cmd {
            TimerCmd::PromptTransitionAnsweredYes => {
                ret.extend(self.update(Msg::Pomodoro(PomodoroMsg::NextSession)))
            }
            TimerCmd::PromptTransitionAnsweredNo => {
                ret.extend(self.update(Msg::Pomodoro(PomodoroMsg::NextSession)));
                ret.extend(self.update(Msg::Pomodoro(PomodoroMsg::Pause)))
            }
        };
        self.set_overlay(None);

        ret
    }

    fn translate_settings_cmd(&mut self, cmd: SettingsCmd) -> Vec<Effect> {
        let mut ret = vec![];
        match cmd {
            SettingsCmd::SaveEdit(msg) => {
                ret.extend(self.update(Msg::Config(msg)));
            }
            SettingsCmd::SaveConfig => {
                ret.push(Effect::SaveConfig(Box::new(self.config().clone())));
                self.update_config_snapshot();
            }
            SettingsCmd::ShowToast { message, r#type } => {
                ret.push(Effect::ShowToast {
                    message,
                    kind: r#type,
                });
            }
        }
        ret
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
    use crate::config::Config;
    use crate::model::Mode;
    use crate::model::Pomodoro;
    use crate::ui::core::Overlay;
    struct MockEffects;
    impl EffectHandler for MockEffects {
        fn execute(&mut self, _cmd: Effect) -> Vec<Msg> {
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
        let core = AppCore::new(test_pomodoro(), test_config(), MockEffects, true, None);
        assert_eq!(core.overlay(), Some(Overlay::DuplicateWarning));
    }

    #[test]
    fn no_duplicate_no_overlay() {
        let core = AppCore::new(test_pomodoro(), test_config(), MockEffects, false, None);
        assert_eq!(core.overlay(), None);
    }

    #[test]
    fn tick_updates_active_session() {
        let mut core = AppCore::new(test_pomodoro(), test_config(), MockEffects, false, None);
        let ses = Session {
            id: 42,
            ..Default::default()
        };
        core.update(Msg::SessionResult(SessionResultMsg::Added(ses.clone())));

        let cmds = core.update(Msg::Tick);

        assert!(
            cmds.iter()
                .any(|c| matches!(c, Effect::Session(SessionEffect::Update { id: 42 })))
        );
    }

    #[test]
    fn tick_without_session() {
        let mut core = AppCore::new(test_pomodoro(), test_config(), MockEffects, false, None);
        let cmds = core.update(Msg::Tick);

        assert!(
            !cmds
                .iter()
                .any(|c| matches!(c, Effect::Session(SessionEffect::Update { .. })))
        );
    }

    #[test]
    fn quit_with_dirty_config_shows_warning() {
        let mut core = AppCore::new(test_pomodoro(), test_config(), MockEffects, false, None);
        let _ = core.update(Msg::Config(ConfigMsg::TimerAutoFocus));

        let cmds = core.update(Msg::Quit);

        assert_eq!(core.overlay(), Some(Overlay::UnsavedWarning));
        assert!(!core.is_quit());
        assert!(!cmds.iter().any(|c| matches!(c, Effect::Quit)));
    }

    #[test]
    fn quit_with_clean_config_exits() {
        let mut core = AppCore::new(test_pomodoro(), test_config(), MockEffects, false, None);
        let cmds = core.update(Msg::Quit);

        assert!(core.is_quit());
        assert!(cmds.iter().any(|c| matches!(c, Effect::Quit)));
        assert_eq!(core.overlay(), None);
    }

    #[test]
    fn overlay_dismiss_messages() {
        let mut core = AppCore::new(test_pomodoro(), test_config(), MockEffects, false, None);

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
        let mut core = AppCore::new(test_pomodoro(), test_config(), MockEffects, false, None);
        core.set_overlay(Some(Overlay::ResetWarning));

        let _cmds = core.update(Msg::ResetWarningProceed);

        assert_eq!(core.overlay(), None);
    }

    #[test]
    fn reset_warning_show() {
        let mut core = AppCore::new(test_pomodoro(), test_config(), MockEffects, false, None);
        assert_eq!(core.overlay(), None);

        core.update(Msg::ResetWarningShow);
        assert_eq!(core.overlay(), Some(Overlay::ResetWarning));
    }

    #[test]
    fn unsaved_warning_save_and_quit() {
        let mut core = AppCore::new(test_pomodoro(), test_config(), MockEffects, false, None);
        core.set_overlay(Some(Overlay::UnsavedWarning));

        let cmds = core.update(Msg::UnsavedWarningSave);

        assert_eq!(core.overlay(), None);
        assert!(cmds.iter().any(|c| matches!(c, Effect::SaveConfig(_))));
        assert!(cmds.iter().any(|c| matches!(c, Effect::Quit)));
    }

    #[test]
    fn unsaved_warning_quit() {
        let mut core = AppCore::new(test_pomodoro(), test_config(), MockEffects, false, None);
        core.set_overlay(Some(Overlay::UnsavedWarning));

        core.update(Msg::UnsavedWarningQuit);
        assert!(core.is_quit());
    }

    #[test]
    fn duplicate_warning_quit() {
        let mut core = AppCore::new(test_pomodoro(), test_config(), MockEffects, false, None);
        core.set_overlay(Some(Overlay::DuplicateWarning));

        let cmds = core.update(Msg::DuplicateWarningQuit);
        assert!(cmds.iter().any(|c| matches!(c, Effect::Quit)));
    }

    #[test]
    fn config_saved_ok() {
        let mut core = AppCore::new(test_pomodoro(), test_config(), MockEffects, false, None);
        let _ = core.update(Msg::Config(ConfigMsg::TimerAutoFocus));
        assert!(core.is_config_dirty());

        let cmds = core.update(Msg::ConfigSaved(ConfigSaveResult::Ok));

        assert!(!core.is_config_dirty());
        assert!(cmds.iter().any(|c| matches!(
            c,
            Effect::ShowToast {
                kind: ToastType::Success,
                ..
            }
        )));
    }

    #[test]
    fn config_saved_err() {
        let mut core = AppCore::new(test_pomodoro(), test_config(), MockEffects, false, None);
        let _ = core.update(Msg::Config(ConfigMsg::TimerAutoFocus));
        assert!(core.is_config_dirty());

        let cmds = core.update(Msg::ConfigSaved(ConfigSaveResult::Err("disk full".into())));

        assert!(core.is_config_dirty());
        assert!(cmds.iter().any(|c| matches!(
            c,
            Effect::ShowToast {
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
        let mut core = AppCore::new(pomo, test_config(), MockEffects, false, None);

        core.update(Msg::Pomodoro(PomodoroMsg::Start));
        core.update(Msg::SessionResult(SessionResultMsg::Added(Session {
            id: 1,
            ..Default::default()
        })));

        let cmds = core.update(Msg::Pomodoro(PomodoroMsg::Tick));

        assert!(
            cmds.iter()
                .any(|c| matches!(c, Effect::SendNotification(_)))
        );
        assert!(cmds.iter().any(|c| matches!(c, Effect::PlaySound(_))));
        assert!(cmds.iter().any(|c| matches!(c, Effect::RunHook(_))));
        assert!(
            cmds.iter()
                .any(|c| matches!(c, Effect::Session(SessionEffect::End { id: 1 })))
        );
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

        let mut core = AppCore::new(pomo, config, MockEffects, false, None);
        core.update(Msg::Pomodoro(PomodoroMsg::Start));
        core.update(Msg::SessionResult(SessionResultMsg::Added(Session {
            id: 1,
            ..Default::default()
        })));

        let cmds = core.update(Msg::Pomodoro(PomodoroMsg::Tick));

        assert!(
            cmds.iter()
                .any(|c| matches!(c, Effect::SendNotification(_)))
        );
        assert!(cmds.iter().any(|c| matches!(c, Effect::PlaySound(_))));
        assert!(cmds.iter().any(|c| matches!(c, Effect::RunHook(_))));
        assert_eq!(core.overlay(), None);
        assert!(
            cmds.iter()
                .any(|c| matches!(c, Effect::Session(SessionEffect::Add { .. })))
        );
    }

    #[test]
    fn session_end_idempotent_effects() {
        let pomo = Pomodoro::new(
            Duration::from_secs(0),
            Duration::from_mins(10),
            Duration::from_mins(5),
            4,
        );
        let mut core = AppCore::new(pomo, test_config(), MockEffects, false, None);

        core.update(Msg::Pomodoro(PomodoroMsg::Start));
        core.update(Msg::SessionResult(SessionResultMsg::Added(Session {
            id: 1,
            ..Default::default()
        })));

        // First session end fires effects
        core.update(Msg::Pomodoro(PomodoroMsg::Tick));
        assert_eq!(core.overlay(), Some(Overlay::PromptingTransition));

        // Another tick while prompting should not fire duplicate effects
        core.update(Msg::SessionResult(SessionResultMsg::Added(Session {
            id: 2,
            ..Default::default()
        })));
        let cmds = core.update(Msg::Pomodoro(PomodoroMsg::Tick));
        assert!(
            !cmds
                .iter()
                .any(|c| matches!(c, Effect::SendNotification(_)))
        );
        assert!(!cmds.iter().any(|c| matches!(c, Effect::PlaySound(_))));
        assert!(!cmds.iter().any(|c| matches!(c, Effect::RunHook(_))));
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

    // ---------------------------------------------------------
    //  _____ ___    _   _  _ ___ _      _ _____ ___ ___  _  _
    // |_   _| _ \  /_\ | \| / __| |    /_\_   _|_ _/ _ \| \| |
    //   | | |   / / _ \| .` \__ \ |__ / _ \| |  | | (_) | .` |
    //   |_| |_|_\/_/ \_\_|\_|___/____/_/ \_\_| |___\___/|_|\_|
    //
    //  _____ ___ ___ _____ ___
    // |_   _| __/ __|_   _/ __|
    //   | | | _|\__ \ | | \__ \
    //   |_| |___|___/ |_| |___/
    // ---------------------------------------------------------

    fn test_appcore() -> AppCore<MockEffects> {
        AppCore::new(
            Pomodoro::default(),
            Config::new(PathBuf::from("/tmp/test_tomo_config")),
            MockEffects,
            false,
            None,
        )
    }

    #[test]
    fn pomodoro_cmd_started_to_new_session() {
        let mut core = test_appcore();
        let cmds = core.translate_pomodoro_cmd(PomodoroCmd::Started(Mode::Focus));

        assert_eq!(cmds.len(), 1);
        assert!(matches!(
            cmds[0],
            Effect::Session(SessionEffect::Add { .. })
        ));
    }

    #[test]
    fn pomodoro_cmd_started_paused_empty() {
        let mut core = test_appcore();
        let cmds = core.translate_pomodoro_cmd(PomodoroCmd::StartedPaused);

        assert!(cmds.is_empty());
    }

    #[test]
    fn pomodoro_cmd_session_paused_with_active_session() {
        let mut core = test_appcore();
        core.update(Msg::SessionResult(SessionResultMsg::Added(Session {
            id: 7,
            ..Default::default()
        })));

        let cmds = core.translate_pomodoro_cmd(PomodoroCmd::SessionPaused);

        assert_eq!(cmds.len(), 1);
        assert!(matches!(
            cmds[0],
            Effect::Session(SessionEffect::End { id: 7 })
        ));
    }

    #[test]
    fn pomodoro_cmd_session_paused_without_active_session() {
        let mut core = test_appcore();
        let cmds = core.translate_pomodoro_cmd(PomodoroCmd::SessionPaused);

        assert!(cmds.is_empty());
    }

    #[test]
    fn pomodoro_cmd_session_resumed_to_new_session() {
        let mut core = test_appcore();
        let cmds = core.translate_pomodoro_cmd(PomodoroCmd::SessionResumed(Mode::Focus));

        assert_eq!(cmds.len(), 1);
        assert!(matches!(
            cmds[0],
            Effect::Session(SessionEffect::Add { .. })
        ));
    }

    #[test]
    fn pomodoro_cmd_session_skipped_ends_and_starts() {
        let mut core = test_appcore();
        core.update(Msg::SessionResult(SessionResultMsg::Added(Session {
            id: 3,
            ..Default::default()
        })));

        let cmds = core.translate_pomodoro_cmd(PomodoroCmd::SessionSkipped(Mode::ShortBreak));

        assert_eq!(cmds.len(), 2);
        assert!(matches!(
            cmds[0],
            Effect::Session(SessionEffect::End { id: 3 })
        ));
        assert!(matches!(
            cmds[1],
            Effect::Session(SessionEffect::Add { .. })
        ));
    }

    #[test]
    fn pomodoro_cmd_next_session_to_new_session() {
        let mut core = test_appcore();
        let cmds = core.translate_pomodoro_cmd(PomodoroCmd::NextSession(Mode::LongBreak));

        assert_eq!(cmds.len(), 1);
        assert!(matches!(
            cmds[0],
            Effect::Session(SessionEffect::Add { .. })
        ));
    }

    #[test]
    fn timer_cmd_yes_clears_overlay() {
        let mut core = test_appcore();
        core.set_overlay(Some(Overlay::PromptingTransition));

        core.translate_timer_cmd(TimerCmd::PromptTransitionAnsweredYes);

        assert_eq!(core.overlay(), None);
    }

    #[test]
    fn timer_cmd_no_clears_overlay() {
        let mut core = test_appcore();
        core.set_overlay(Some(Overlay::PromptingTransition));

        core.translate_timer_cmd(TimerCmd::PromptTransitionAnsweredNo);

        assert_eq!(core.overlay(), None);
    }

    #[test]
    fn timer_cmd_starts_with_stop_sound() {
        let mut core = test_appcore();
        let cmds = core.translate_timer_cmd(TimerCmd::PromptTransitionAnsweredYes);

        assert!(matches!(cmds[0], Effect::StopSound));
    }

    #[test]
    fn settings_cmd_save_config() {
        let mut core = test_appcore();
        let cmds = core.translate_settings_cmd(SettingsCmd::SaveConfig);

        assert_eq!(cmds.len(), 1);
        assert!(matches!(cmds[0], Effect::SaveConfig(_)));
        assert!(!core.is_config_dirty());
    }

    #[test]
    fn settings_cmd_show_toast() {
        let mut core = test_appcore();
        let cmds = core.translate_settings_cmd(SettingsCmd::ShowToast {
            message: "hello".into(),
            r#type: ToastType::Warning,
        });

        assert_eq!(cmds.len(), 1);
        assert!(matches!(
            &cmds[0],
            Effect::ShowToast {
                kind: ToastType::Warning,
                ..
            }
        ));
    }

    #[test]
    fn config_cmd_none_empty() {
        let mut core = test_appcore();
        let cmds = core.translate_config_cmd(ConfigCmd::None);

        assert!(cmds.is_empty());
    }

    #[test]
    fn router_cmd_none_empty() {
        let mut core = test_appcore();
        let cmds = core.translate_router_cmd(RouterCmd::None);

        assert!(cmds.is_empty());
    }
}
