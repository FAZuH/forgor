use chrono::NaiveDateTime;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Task {
    pub id: i32,
    pub name: String,
    pub description: Option<String>,
    pub deadline: Option<NaiveDateTime>,
    pub parent_id: Option<i32>,
    pub project_id: Option<i32>,
}
