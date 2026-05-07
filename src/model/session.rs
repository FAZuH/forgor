use chrono::NaiveDateTime;

use super::Mode;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Session {
    pub id: i32,
    pub task_id: Option<i32>,
    pub start_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub end_at: Option<NaiveDateTime>,
    pub mode: Mode,
    pub paused: bool,
}

impl Default for Session {
    fn default() -> Self {
        Self {
            id: Default::default(),
            task_id: Default::default(),
            start_at: Default::default(),
            updated_at: Default::default(),
            end_at: Default::default(),
            mode: Mode::Focus,
            paused: Default::default(),
        }
    }
}
