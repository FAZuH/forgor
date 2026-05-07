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
    StartTaskPrompt,
    CancelTaskPrompt,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TimerCmd {
    PromptTransitionAnsweredYes,
    PromptTransitionAnsweredNo,
    FetchAllTasks,
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;

    fn test_pomodoro() -> Pomodoro {
        Pomodoro::new(
            Duration::from_mins(25),
            Duration::from_mins(10),
            Duration::from_mins(5),
            4,
        )
    }

    #[test]
    fn toggle_pause_when_running() {
        let mut pomo = test_pomodoro();
        pomo.start().unwrap();

        let cmds = pomo.update(PomodoroMsg::TogglePause);

        assert_eq!(cmds.len(), 1);
        assert!(matches!(cmds[0], PomodoroCmd::SessionPaused));
        assert!(!pomo.is_running());
    }

    #[test]
    fn toggle_pause_when_paused() {
        let mut pomo = test_pomodoro();
        pomo.start().unwrap();
        pomo.pause().unwrap();

        let cmds = pomo.update(PomodoroMsg::TogglePause);

        assert_eq!(cmds.len(), 1);
        assert!(matches!(cmds[0], PomodoroCmd::SessionResumed(_)));
        assert!(pomo.is_running());
    }

    #[test]
    fn start_emits_started() {
        let mut pomo = test_pomodoro();
        let cmds = pomo.update(PomodoroMsg::Start);

        assert!(cmds.iter().any(|c| matches!(c, PomodoroCmd::Started(_))));
        assert!(pomo.is_running());
    }

    #[test]
    fn start_paused_emits_started_paused() {
        let mut pomo = test_pomodoro();
        let cmds = pomo.update(PomodoroMsg::StartPaused);

        assert_eq!(cmds.len(), 1);
        assert!(matches!(cmds[0], PomodoroCmd::StartedPaused));
        assert!(!pomo.is_running());
    }

    #[test]
    fn add_increases_remaining() {
        let mut pomo = test_pomodoro();
        pomo.update(PomodoroMsg::Start);
        let before = pomo.remaining_time();

        pomo.update(PomodoroMsg::Add(Duration::from_secs(30)));
        assert!(pomo.remaining_time() > before);
    }

    #[test]
    fn subtract_decreases_remaining() {
        let mut pomo = test_pomodoro();
        pomo.update(PomodoroMsg::Start);
        let before = pomo.remaining_time();

        pomo.update(PomodoroMsg::Subtract(Duration::from_secs(30)));
        assert!(pomo.remaining_time() < before);
    }

    #[test]
    fn tick_noop_when_time_remaining() {
        let mut pomo = test_pomodoro();
        pomo.update(PomodoroMsg::Start);

        let cmds = pomo.update(PomodoroMsg::Tick);
        assert!(cmds.is_empty());
    }

    #[test]
    fn tick_session_end_when_time_elapsed() {
        let mut pomo = Pomodoro::new(
            Duration::from_secs(0),
            Duration::from_mins(10),
            Duration::from_mins(5),
            4,
        );
        pomo.start().unwrap();

        let cmds = pomo.update(PomodoroMsg::Tick);

        assert_eq!(cmds.len(), 1);
        assert!(matches!(cmds[0], PomodoroCmd::SessionEnd { .. }));
    }

    #[test]
    fn skip_emits_session_skipped() {
        let mut pomo = test_pomodoro();
        pomo.update(PomodoroMsg::Start);

        let cmds = pomo.update(PomodoroMsg::SkipSession);

        assert_eq!(cmds.len(), 1);
        assert!(matches!(cmds[0], PomodoroCmd::SessionSkipped(_)));
    }

    #[test]
    fn next_session_emits_next_session() {
        let mut pomo = test_pomodoro();
        pomo.update(PomodoroMsg::Start);

        let cmds = pomo.update(PomodoroMsg::NextSession);

        assert_eq!(cmds.len(), 1);
        assert!(matches!(cmds[0], PomodoroCmd::NextSession(_)));
    }

    #[test]
    fn pause_emits_session_paused() {
        let mut pomo = test_pomodoro();
        pomo.start().unwrap();

        let cmds = pomo.update(PomodoroMsg::Pause);

        assert_eq!(cmds.len(), 1);
        assert!(matches!(cmds[0], PomodoroCmd::SessionPaused));
        assert!(!pomo.is_running());
    }

    #[test]
    fn resume_emits_session_resumed() {
        let mut pomo = test_pomodoro();
        pomo.start().unwrap();
        pomo.pause().unwrap();

        let cmds = pomo.update(PomodoroMsg::Resume);

        assert_eq!(cmds.len(), 1);
        assert!(matches!(cmds[0], PomodoroCmd::SessionResumed(_)));
        assert!(pomo.is_running());
    }

    #[test]
    fn reset_session_returns_empty() {
        let mut pomo = test_pomodoro();
        pomo.start().unwrap();

        let cmds = pomo.update(PomodoroMsg::ResetSession);

        assert!(cmds.is_empty());
    }
}
