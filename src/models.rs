#[derive(Queryable)]
pub struct Project {
    pub id: i64,
    pub code: String,
    pub directory: Option<String>,
    pub name: Option<String>,
}

#[derive(Queryable)]
pub struct Stretch {
    pub id: i64,
    pub subtask_id: i64,
    pub start: i64,
    pub end: Option<i64>,
}

#[derive(Queryable)]
pub struct Subtask {
    pub id: i64,
    pub task_id: i64,
    pub branch: Option<String>,
    pub description: Option<String>,
    pub active: bool,
}

#[derive(Queryable)]
pub struct Task {
    pub id: i64,
    pub project_id: i64,
    pub number: i64,
    pub active_subtask: Option<i64>,
}

