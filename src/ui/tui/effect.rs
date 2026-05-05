use crate::config::Alarm;
use crate::repo::Repos;
use crate::service::NotifyService;
use crate::service::SoundService;
use crate::service::cmd_runner::run_hook_command;
use crate::ui::prelude::*;
use crate::ui::tui::toast::ToastHandler;

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
    fn execute(&mut self, cmd: Cmd) -> Vec<Msg> {
        let mut ret = Vec::new();
        match cmd {
            Cmd::PlaySound(alarm) => {
                if !self.sound.is_playing() {
                    self.sound.set_sound(alarm);
                    let _ = self.sound.play();
                }
            }
            Cmd::StopSound => {
                let _ = self.sound.stop();
            }
            Cmd::SendNotification(mode) => {
                let result = self.notify.send(mode);
                ret.push(Msg::NotificationSent(result));
            }
            Cmd::NewSession { task_id, state } => {
                let session = self.repo.session().new_session(task_id, state).unwrap();
                ret.push(Msg::SessionCreated { id: session.id });
            }
            Cmd::UpdateSession { id } => {
                let _ = self.repo.session().update(id);
                ret.push(Msg::SessionUpdated);
            }
            Cmd::EndSession { id } => {
                let _ = self.repo.session().end_session(id);
                ret.push(Msg::SessionEnded);
            }
            Cmd::CloseAllSessions => {
                let _ = self.repo.session().close_all_sessions();
                ret.push(Msg::SessionsClosed);
            }
            Cmd::SaveConfig(config) => {
                let result: Result<(), String> = config.save().map_err(|e| e.to_string());
                ret.push(Msg::ConfigSaved(ConfigSaveResult::from(result)));
            }
            Cmd::RunHook(command) => {
                run_hook_command(&command);
            }
            Cmd::ShowToast { message, kind } => {
                self.toast.show(message, kind);
            }
        }
        ret
    }
}
