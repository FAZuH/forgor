use std::time::Duration;

use crate::model::Pomodoro;
use crate::ui::prelude::*;

#[derive(Clone, Debug, Copy, PartialEq, Eq)]
pub enum PomodoroMsg {
    Add(Duration),
    Subtract(Duration),
    TogglePause,
    Pause,
    Resume,
    SkipSession,
    ResetSession,
    Tick { auto_next: bool },
    NextState,
}

#[derive(Clone, Debug, Copy, PartialEq, Eq)]
pub enum PomodoroCmd {
    None,
    PromptNextSession,
    NextSession,
    SessionContinued,
}

impl Updateable for Pomodoro {
    type Msg = PomodoroMsg;
    type Cmd = PomodoroCmd;

    fn update(&mut self, msg: Self::Msg) -> Vec<Self::Cmd> {
        use PomodoroMsg::*;
        let mut cmd = vec![];
        match msg {
            Add(dur) => self.add(dur),
            Subtract(dur) => self.subtract(dur),
            TogglePause => self.toggle_pause(),
            Pause => {
                let _ = self.pause();
            }
            Resume => {
                let _ = self.resume();
            }
            SkipSession => self.skip(),
            ResetSession => self.reset(),
            NextState => {
                self.skip();
                cmd.push(PomodoroCmd::SessionContinued)
            }
            Tick { auto_next } => cmd.push(self.tick(auto_next)),
        }
        cmd
    }
}

impl Pomodoro {
    pub fn tick(&mut self, auto_next: bool) -> PomodoroCmd {
        if self.remaining_time().is_zero() {
            if auto_next {
                PomodoroCmd::NextSession
            } else {
                PomodoroCmd::PromptNextSession
            }
        } else {
            PomodoroCmd::None
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimerMsg {
    SetPromptNextSession(bool),
    SetShowKeybinds(bool),
    ToggleShowKeybinds,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimerCmd {
    None,
}
