use crate::ui::prelude::*;

impl<E: EffectHandler> AppCore<E> {
    pub(crate) fn translate_pomodoro_cmd(&mut self, cmd: PomodoroCmd) -> Vec<Cmd> {
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
                if let Some(id) = self.take_active_session_id() {
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
                if let Some(id) = self.take_active_session_id() {
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

    pub(crate) fn translate_config_cmd(&mut self, cmd: ConfigCmd) -> Vec<Cmd> {
        let ret = vec![];
        match cmd {
            ConfigCmd::None => {}
        }
        ret
    }

    pub(crate) fn translate_router_cmd(&mut self, cmd: RouterCmd) -> Vec<Cmd> {
        let ret = vec![];
        match cmd {
            RouterCmd::None => {}
        }
        ret
    }

    pub(crate) fn translate_timer_cmd(&mut self, cmd: TimerCmd) -> Vec<Cmd> {
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
        self.set_overlay(None);

        ret
    }

    pub(crate) fn translate_settings_cmd(&mut self, cmd: SettingsCmd) -> Vec<Cmd> {
        let mut ret = vec![];
        match cmd {
            SettingsCmd::SaveEdit(msg) => {
                ret.extend(self.update(Msg::Config(msg)));
            }
            SettingsCmd::SaveConfig => {
                ret.push(Cmd::SaveConfig(Box::new(self.config().clone())));
                self.update_config_snapshot();
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

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;
    use crate::config::Config;
    use crate::model::Mode;
    use crate::model::Pomodoro;
    use crate::ui::core::Overlay;

    struct MockEffects;
    impl EffectHandler for MockEffects {
        fn execute(&mut self, _cmd: Cmd) -> Vec<Msg> {
            vec![]
        }
    }

    fn test_appcore() -> AppCore<MockEffects> {
        AppCore::new(
            Pomodoro::default(),
            Config::new(PathBuf::from("/tmp/test_tomo_config")),
            MockEffects,
            false,
        )
    }

    #[test]
    fn pomodoro_cmd_started_to_new_session() {
        let mut core = test_appcore();
        let cmds = core.translate_pomodoro_cmd(PomodoroCmd::Started(Mode::Focus));

        assert_eq!(cmds.len(), 1);
        assert!(matches!(cmds[0], Cmd::NewSession { .. }));
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
        core.update(Msg::SessionCreated { id: 7 });

        let cmds = core.translate_pomodoro_cmd(PomodoroCmd::SessionPaused);

        assert_eq!(cmds.len(), 1);
        assert!(matches!(cmds[0], Cmd::EndSession { id: 7 }));
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
        assert!(matches!(cmds[0], Cmd::NewSession { .. }));
    }

    #[test]
    fn pomodoro_cmd_session_skipped_ends_and_starts() {
        let mut core = test_appcore();
        core.update(Msg::SessionCreated { id: 3 });

        let cmds = core.translate_pomodoro_cmd(PomodoroCmd::SessionSkipped(Mode::ShortBreak));

        assert_eq!(cmds.len(), 2);
        assert!(matches!(cmds[0], Cmd::EndSession { id: 3 }));
        assert!(matches!(cmds[1], Cmd::NewSession { .. }));
    }

    #[test]
    fn pomodoro_cmd_next_session_to_new_session() {
        let mut core = test_appcore();
        let cmds = core.translate_pomodoro_cmd(PomodoroCmd::NextSession(Mode::LongBreak));

        assert_eq!(cmds.len(), 1);
        assert!(matches!(cmds[0], Cmd::NewSession { .. }));
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

        assert!(matches!(cmds[0], Cmd::StopSound));
    }

    #[test]
    fn settings_cmd_save_config() {
        let mut core = test_appcore();
        let cmds = core.translate_settings_cmd(SettingsCmd::SaveConfig);

        assert_eq!(cmds.len(), 1);
        assert!(matches!(cmds[0], Cmd::SaveConfig(_)));
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
            Cmd::ShowToast {
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
