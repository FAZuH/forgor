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
