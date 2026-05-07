use crate::model::Mode;
use crate::model::Session;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SessionMsg {
    Add {
        name: String,
        description: Option<String>,
    },
    Select(Session),
    ClearSelection,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SessionEffect {
    Add { task_id: Option<i32>, mode: Mode },
    Update { id: i32 },
    End { id: i32 },
    EndAll,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SessionResultMsg {
    Added(Session),
    Ended,
}
