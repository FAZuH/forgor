use notify_rust::Notification;
use notify_rust::{self};

use crate::model::Mode;
use crate::service::NotifyService;

impl From<Mode> for Notification {
    fn from(value: Mode) -> Self {
        let (summary, body) = match value {
            Mode::Focus => ("Time to focus!", "Your break is over. Get back to work."),
            Mode::LongBreak => ("Long break time!", "You've earned it. Take a long break."),
            Mode::ShortBreak => ("Short break time!", "Take a quick breather."),
        };

        let mut ret = Notification::new();
        ret.summary(summary).body(body);
        ret
    }
}

pub struct DesktopNotifyService;

impl NotifyService for DesktopNotifyService {
    fn send(&mut self, mode: Mode) -> Result<(), String> {
        let notification: Notification = mode.into();
        notification.show().map(|_| ()).map_err(|e| e.to_string())
    }
}
