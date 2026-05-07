use crate::config::Alarm;
use crate::repo::Repos;
use crate::service::NotifyService;
use crate::service::SoundService;
use crate::service::cmd_runner::run_hook_command;
use crate::ui::prelude::*;
use crate::ui::tui::toast::ToastHandler;

/// Executes side-effects for the terminal UI, such as desktop notifications or audio playback.
pub struct TuiEffectHandler {
    toast: ToastHandler,
    sound: Box<dyn SoundService<SoundType = Alarm>>,
    notify: Box<dyn NotifyService>,
    repo: Box<dyn Repos>,
}

impl TuiEffectHandler {
    pub fn new(
        sound: Box<dyn SoundService<SoundType = Alarm>>,
        notify: Box<dyn NotifyService>,
        repo: Box<dyn Repos>,
    ) -> Self {
        Self {
            toast: ToastHandler::new(),
            sound,
            notify,
            repo,
        }
    }

    pub fn toast(&self) -> &ToastHandler {
        &self.toast
    }

    pub fn toast_mut(&mut self) -> &mut ToastHandler {
        &mut self.toast
    }
}

impl EffectHandler for TuiEffectHandler {
    fn execute(&mut self, cmd: Effect) -> Vec<Msg> {
        let mut ret = Vec::new();
        match cmd {
            Effect::PlaySound(alarm) => {
                if !self.sound.is_playing() {
                    self.sound.set_sound(alarm);
                    let _ = self.sound.play();
                }
            }
            Effect::StopSound => {
                let _ = self.sound.stop();
            }
            Effect::SendNotification(mode) => {
                let result = self.notify.send(mode);
                ret.push(Msg::NotificationSent(result));
            }
            Effect::SaveConfig(config) => {
                let result: Result<(), String> = config.save().map_err(|e| e.to_string());
                ret.push(Msg::ConfigSaved(ConfigSaveResult::from(result)));
            }
            Effect::RunHook(command) => {
                run_hook_command(&command);
            }
            Effect::ShowToast { message, kind } => {
                self.toast.show(message, kind);
            }
            Effect::Quit => {
                let _ = self.sound.stop();
            }
            Effect::Session(eff) => {
                use SessionEffect::*;
                // TODO: Handle errs
                match eff {
                    Add { task_id, mode } => {
                        let session = self.repo.session().new_session(task_id, mode).unwrap();
                        ret.push(Msg::SessionResult(SessionResultMsg::Added(session)));
                    }
                    Update { id } => {
                        let _ = self.repo.session().update(id);
                        // ret.push(Msg::SessionResult(SessionResultMsg::Updated));
                    }
                    End { id } => {
                        let _ = self.repo.session().end_session(id);
                        // ret.push(Msg::SessionResult(SessionResultMsg::Ended));
                    }
                    EndAll => {
                        let _ = self.repo.session().close_all_sessions();
                        // ret.push(Msg::SessionResult(SessionResultMsg::ClosedAll));
                    }
                }
            }
            Effect::Task(eff) => match eff {
                TaskEffect::Add { name, description } => {
                    match self.repo.task().add(name, description) {
                        Ok(task) => ret.push(Msg::TaskResult(TaskResultMsg::Added(task))),
                        Err(e) => ret.extend(self.execute(Effect::ShowToast {
                            message: format!("Error when creating task: {e}"),
                            kind: ToastType::Error,
                        })),
                    }
                }
            },
        }
        ret
    }
}
