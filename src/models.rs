use diesel::sqlite::SqliteConnection;
use crate::diesel::QueryDsl;
use crate::diesel::RunQueryDsl;
use crate::diesel::ExpressionMethods;

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

pub fn get_project<'a>(conn: &SqliteConnection, code: &'a str) -> Result<Project, diesel::result::Error> {
    use super::schema::projects;
    use super::schema::projects::dsl;
    #[derive(Insertable)]
    #[table_name="projects"]
    struct NewProject<'x> {
        code: &'x str,
    }
    let new_project = NewProject {
        code: code,
    };
    SqliteConnection::immediate_transaction(conn, || {
        loop {
            let project = dsl::projects.filter(dsl::code.eq(code))
                .limit(1)
                .get_result::<Project>(conn);
            match project {
                Err(diesel::result::Error::NotFound) => {},
                _ => break project
            }
            diesel::insert_into(projects::table)
                .values(&new_project)
                .execute(conn)?;
        }
    })
}

