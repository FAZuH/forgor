use notify_rust::Notification;
use notify_rust::NotificationHandle;
use notify_rust::{self};

use crate::models::pomodoro::PomodoroState;

impl From<PomodoroState> for Notification {
    fn from(value: PomodoroState) -> Self {
        let (summary, body) = match value {
            PomodoroState::Focus => ("Time to focus!", "Your break is over. Get back to work."),
            PomodoroState::LongBreak => {
                ("Long break time!", "You've earned it. Take a long break.")
            }
            PomodoroState::ShortBreak => ("Short break time!", "Take a quick breather."),
        };

        let mut ret = Notification::new();
        ret.summary(summary).body(body);
        ret
    }
}

pub fn notify(
    notifiable: impl Into<Notification>,
) -> Result<NotificationHandle, notify_rust::error::Error> {
    notifiable.into().show()
}
