use std::time::SystemTime;

use diesel::sqlite::{Sqlite, SqliteConnection};
use diesel::{Connection, QueryDsl, RunQueryDsl, ExpressionMethods};
use chrono::DateTime;
use chrono::offset::TimeZone;
use chrono_tz::Tz;
use chrono_tz::Pacific::Auckland;
use super::schema;

#[derive(Queryable)]
pub struct Project {
    pub id: i64,
    pub code: String,
    pub directory: Option<String>,
    pub name: Option<String>,
}

pub struct Stretch {
    pub id: i64,
    pub subtask_id: i64,
    pub start: DateTime<Tz>,
    pub end: Option<DateTime<Tz>>,
}

impl diesel::deserialize::Queryable<super::schema::stretches::SqlType, diesel::sqlite::Sqlite> for Stretch {
    type Row = (i64, i64, i64, Option<i64>);

    fn build(row: Self::Row) -> Self {
        Stretch {
            id: row.0,
            subtask_id: row.1,
            start: Auckland.timestamp(row.2, 0),
            end: row.3.map(|ts| Auckland.timestamp(ts, 0))
        }
    }
}

#[derive(Queryable)]
pub struct Subtask {
    pub id: i64,
    pub task_id: i64,
    pub branch: Option<String>,
    pub description: Option<String>,
    pub active: bool,
    pub number: i64,
}

#[derive(Queryable)]
pub struct Task {
    pub id: i64,
    pub project_id: i64,
    pub number: i64,
    pub active_subtask: Option<i64>,
}

pub struct SubtaskSpec {
    pub project_code: String,
    pub task_number: i64,
    pub subtask_number: Option<i64>,
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
    SqliteConnection::transaction(conn, || {
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
        use schema::tasks::dsl;
        use schema::tasks;
        #[derive(Insertable)]
        #[table_name="tasks"]
        struct NewTask {
            project_id: i64,
            number: i64,
        }
        SqliteConnection::transaction(conn, || loop {
            let task = dsl::tasks
                .filter(dsl::project_id.eq(self.id))
                .filter(dsl::number.eq(number))
                .get_result::<Task>(conn);
            match task {
                Err(diesel::result::Error::NotFound) => {},
                _ => break task
            }
            diesel::insert_into(tasks::table)
                .values(&NewTask {
                    project_id: self.id,
                    number: number,
                }).execute(conn)?;
        })
    }
}

impl Task {
    pub fn code(&self, conn: &SqliteConnection) -> String {
        use schema::projects::dsl;
        let project_code = dsl::projects.filter(dsl::id.eq(self.project_id))
            .select(dsl::code)
            .get_result::<String>(conn)
            .ok()
            .unwrap_or_else(|| String::from("????"));
        format!("{}-{}", project_code, self.number)
    }

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

    pub fn current(conn: &SqliteConnection) -> Option<Self> {
        use schema::tasks::dsl;
        let q = 
        current_stretch_scope(
            dsl::tasks.inner_join(
                schema::subtasks::dsl::subtasks
                .inner_join(schema::stretches::dsl::stretches)
            )).order(schema::stretches::dsl::start.desc())
            .select(schema::tasks::all_columns);
        println!("{}", diesel::query_builder::debug_query::<Sqlite, _>(&q));
        q
            .get_result::<Self>(conn)
            .ok()
    }
}

impl std::str::FromStr for SubtaskSpec {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut split = s.split('-');
        Ok(SubtaskSpec {
            project_code: split.next()
                .ok_or_else(|| String::from("Failed to parse code"))
                .map(String::from)?,
            task_number: split.next()
                .ok_or_else(|| String::from("Missing task number"))
                .and_then(|part| part.parse().map_err(|e| format!("{}", e)))?,
            subtask_number: split.next()
                .map(|part| part.parse().map_err(|e| format!("{}",e)))
                .transpose()?,
        })
    }
}

impl Subtask {
    pub fn for_code(conn: &SqliteConnection, code: &str) -> Result<(Project,Task,Self), String> {
        let spec: SubtaskSpec = code.parse()?;
        SqliteConnection::transaction(conn, || {
            let project = get_project(conn, spec.project_code.as_ref())?;
            let task = project.task(conn, spec.task_number)?;
            todo!()
        }).map_err(|e: diesel::result::Error| format!("{}", e))
    }
}

impl Stretch {
    pub fn current(conn: &SqliteConnection) -> Option<Self> {
        use schema::stretches::dsl;
        current_stretch_scope(dsl::stretches)
            .get_result::<Stretch>(conn)
            .ok()
    }

    pub fn stop_all(conn: &SqliteConnection) {
        use schema::stretches::dsl;
        diesel::update(current_stretch_scope(dsl::stretches))
            .set(dsl::end.eq(SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs() as i64))
            .execute(conn)
            .unwrap();
    }
}

fn current_stretch_scope<'a, S: diesel::query_dsl::methods::FilterDsl<diesel::expression::operators::IsNull<schema::stretches::columns::end>>>(scope: S) -> <S as diesel::query_dsl::filter_dsl::FilterDsl<diesel::expression::operators::IsNull<schema::stretches::columns::end>>>::Output {
    use super::schema::stretches::dsl;
    scope.filter(dsl::end.is_null())
}

