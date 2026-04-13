use std::time::Duration;

use PomodoroState::*;

#[derive(Clone, Debug)]
pub struct Pomodoro {
    state: PomodoroState,
    focus: Duration,
    long_break: Duration,
    short_break: Duration,
    long_interval: u32,
    focus_sessions: u32,
}

impl Pomodoro {
    pub fn new(
        focus: Duration,
        long_break: Duration,
        short_break: Duration,
        long_interval: u32,
    ) -> Self {
        Self {
            state: PomodoroState::Focus,
            focus,
            long_break,
            short_break,
            long_interval,
            focus_sessions: 1,
        }
    }

    pub async fn wait(&mut self) {
        let duration = self.get_duration();
        tokio::time::sleep(duration).await;

        self.next_state();
    }

    pub fn next_state(&mut self) {
        match self.state {
            Focus => {
                self.focus_sessions += 1;
                if self.focus_sessions % self.long_interval == 0 {
                    self.state = LongBreak;
                } else {
                    self.state = ShortBreak;
                }
            }
            _ => self.state = Focus,
        }
    }

    fn get_duration(&self) -> Duration {
        let duration = match self.state {
            Focus => self.focus,
            LongBreak => self.long_break,
            ShortBreak => self.short_break,
        };
        duration
    }
}

impl Default for Pomodoro {
    fn default() -> Self {
        Self {
            state: Default::default(),
            focus: Duration::from_mins(25),
            long_break: Duration::from_mins(15),
            short_break: Duration::from_mins(5),
            long_interval: 3,
            focus_sessions: 0,
        }
    }
}

#[derive(Clone, Debug, Copy, PartialEq, Eq)]
pub enum PomodoroState {
    Focus,
    LongBreak,
    ShortBreak,
}

impl Default for PomodoroState {
    fn default() -> Self {
        Self::Focus
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_next_state_focus() {
        let mut pomo = Pomodoro::default();
        pomo.state = ShortBreak;

        pomo.next_state();
        assert_eq!(pomo.state, Focus)
    }

    #[test]
    fn test_next_state_short_break() {
        let mut pomo = Pomodoro::default();

        pomo.next_state();
        assert_eq!(pomo.state, ShortBreak)
    }

    #[test]
    fn test_next_state_long_break() {
        let mut pomo = Pomodoro::default();

        // Short break
        pomo.next_state();
        // Focus
        pomo.next_state();
        // Short break
        pomo.next_state();
        // Focus
        pomo.next_state();

        // Long Break
        pomo.next_state();
        assert_eq!(pomo.state, LongBreak)
    }
}
