use diesel::sqlite::SqliteConnection;
use diesel::{QueryDsl, RunQueryDsl, ExpressionMethods};

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

impl Project {
    pub fn tasks(&self, conn: &SqliteConnection) -> Result<Vec<Task>, diesel::result::Error> {
        use super::schema::tasks::dsl;
        dsl::tasks.filter(dsl::project_id.eq(self.id))
            .load::<Task>(conn)
    }

    pub fn task(&self, conn: &SqliteConnection, number: i64) -> Result<Task, diesel::result::Error> {
        use super::schema::tasks::dsl;
        dsl::tasks
            .filter(dsl::project_id.eq(self.id))
            .filter(dsl::number.eq(number))
            .get_result::<Task>(conn)
    }
}

impl Task {
    pub fn subtasks(&self, conn: &SqliteConnection) -> Result<Vec<Subtask>, diesel::result::Error> {
        use super::schema::subtasks::dsl;
        dsl::subtasks.filter(dsl::task_id.eq(self.id))
            .load::<Subtask>(conn)
    }

    pub fn active_subtask(&self, conn: &SqliteConnection) -> Result<Option<Subtask>, diesel::result::Error> {
        match self.active_subtask {
            None => Ok(None),
            Some(number) => {
                use super::schema::subtasks::dsl;
                match dsl::subtasks.filter(dsl::task_id.eq(self.id))
                    .filter(dsl::id.eq(number))
                    .get_result::<Subtask>(conn) {
                        Err(diesel::result::Error::NotFound) => Ok(None),
                        Err(err) => Err(err),
                        Ok(subtask) => Ok(Some(subtask)),
                    }
            }
        }
    }
}
