use std::time::Duration;

use crate::model::Mode;
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
    Tick,
    NextSession,
    Start,
    StartPaused,
}

#[derive(Clone, Debug, Copy, PartialEq, Eq)]
pub enum PomodoroCmd {
    Started(Mode),
    StartedPaused,

    SessionEnd { curr_mode: Mode, next_mode: Mode },
    NextSession(Mode),

    SessionPaused,
    SessionResumed(Mode),
    SessionSkipped(Mode),
}

impl Updateable<PomodoroMsg, PomodoroCmd> for Pomodoro {
    fn update(&mut self, msg: PomodoroMsg) -> Vec<PomodoroCmd> {
        use PomodoroMsg::*;
        let mut cmds = vec![];
        match msg {
            Start => {
                let _ = self.start();
                cmds.push(PomodoroCmd::Started(self.mode()))
            }
            StartPaused => {
                let _ = self.start();
                let _ = self.pause();
                cmds.push(PomodoroCmd::StartedPaused)
            }
            Add(dur) => self.add(dur),
            Subtract(dur) => self.subtract(dur),
            Pause => {
                let _ = self.pause();
                cmds.push(PomodoroCmd::SessionPaused)
            }
            Resume => {
                let _ = self.resume();
                cmds.push(PomodoroCmd::SessionResumed(self.mode()))
            }
            TogglePause => {
                if self.is_running() {
                    cmds.extend(self.update(Pause));
                } else {
                    cmds.extend(self.update(Resume));
                }
            }
            NextSession => {
                self.skip();
                cmds.push(PomodoroCmd::NextSession(self.mode()))
            }
            SkipSession => {
                self.skip();
                cmds.push(PomodoroCmd::SessionSkipped(self.mode()))
            }
            ResetSession => self.reset(),
            Tick => {
                if self.remaining_time().is_zero() {
                    cmds.push(PomodoroCmd::SessionEnd {
                        curr_mode: self.mode(),
                        next_mode: self.next_mode(),
                    })
                }
            }
        }
        cmds
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimerMsg {
    SetShowKeybinds(bool),
    ToggleShowKeybinds,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimerCmd {
    PromptTransitionAnsweredYes,
    PromptTransitionAnsweredNo,
}
