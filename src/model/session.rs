use chrono::NaiveDateTime;

use super::Mode;

#[derive(Debug, Clone, PartialEq)]
pub struct Session {
    pub id: i32,
    pub task_id: Option<i32>,
    pub start_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub end_at: Option<NaiveDateTime>,
    pub mode: Mode,
    pub paused: bool,
}
