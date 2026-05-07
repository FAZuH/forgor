use crate::model::Task;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TaskMsg {
    Add {
        name: String,
        description: Option<String>,
    },
    Select(Task),
    ClearSelection,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TaskEffect {
    Add {
        name: String,
        description: Option<String>,
    },
    FetchAll,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TaskResultMsg {
    Added(Task),
    FetchedAll(Vec<Task>),
}
