#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tag {
    pub id: i32,
    pub name: String,
    pub description: Option<String>,
}
