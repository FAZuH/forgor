use std::time::Duration;

use crate::models::Pomodoro;
use crate::ui::Updateable;

#[derive(Clone, Debug, Copy, PartialEq, Eq)]
pub enum TimerMsg {
    Add(Duration),
    Subtract(Duration),
    TogglePause,
    SkipSession,
    ResetSession,
    Tick { auto_next: bool },
    NextState,
}

#[derive(Clone, Debug, Copy, PartialEq, Eq)]
pub enum TimerCmd {
    None,
    PromptNextSession,
    NextSession,
    SessionContinued,
}

impl Updateable for Pomodoro {
    type Msg = TimerMsg;
    type Cmd = TimerCmd;

    fn update(&mut self, msg: Self::Msg) -> Self::Cmd {
        use TimerMsg::*;
        let mut cmd = TimerCmd::None;
        match msg {
            Add(dur) => self.add(dur),
            Subtract(dur) => self.subtract(dur),
            TogglePause => self.toggle_pause(),
            SkipSession => self.skip(),
            ResetSession => self.reset(),
            NextState => {
                self.skip();
                cmd = TimerCmd::SessionContinued;
            }
            Tick { auto_next } => cmd = self.tick(auto_next),
        }
        cmd
    }
}

impl Pomodoro {
    pub fn tick(&mut self, auto_next: bool) -> TimerCmd {
        if self.remaining_time().is_zero() {
            if auto_next {
                TimerCmd::NextSession
            } else {
                TimerCmd::PromptNextSession
            }
        } else {
            TimerCmd::None
        }
    }
}
