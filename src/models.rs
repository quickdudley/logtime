use std::time::SystemTime;
use std::collections::HashMap;

use diesel::sqlite::SqliteConnection;
use diesel::{Connection, QueryDsl, RunQueryDsl,
    ExpressionMethods};
use diesel::expression_methods::BoolExpressionMethods;
use chrono::DateTime;
use chrono::Duration;
use chrono::naive::NaiveDate;
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

#[derive(Debug)]
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

    pub fn current(conn: &SqliteConnection) -> Result<Self, diesel::result::Error> {
        use schema::projects::dsl;
        current_stretch_scope(
            dsl::projects.inner_join(
                schema::tasks::dsl::tasks
                .inner_join(
                schema::subtasks::dsl::subtasks
                .inner_join(
                schema::stretches::dsl::stretches
                ))
            )).order(schema::stretches::dsl::start.desc())
            .select(schema::projects::all_columns)
            .get_result::<Self>(conn)
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

    pub fn latest_subtask(&self, conn: &SqliteConnection) -> Result<Subtask, diesel::result::Error> {
        use schema::subtasks::dsl;
        SqliteConnection::transaction(conn, || {
            let my_subtasks = dsl::subtasks
                .filter(dsl::task_id.eq(self.id));
            let latest_number = my_subtasks
                .select(diesel::dsl::max(dsl::number))
                .get_result::<Option<i64>>(conn)
                .and_then(|n| n.ok_or(diesel::result::Error::NotFound))
                .or_else(|e| match e {
                    diesel::result::Error::NotFound => Ok(1),
                    _ => Err(e),
                })?;
            self.load_or_create_subtask(conn, latest_number)
        })
    }

    pub fn load_or_create_subtask(&self, conn: &SqliteConnection, number: i64) -> Result<Subtask, diesel::result::Error> {
        use schema::subtasks;
        SqliteConnection::transaction(conn, || {
            self.subtask(conn, number).or_else(|e| match e {
                diesel::result::Error::NotFound => {
                    #[derive(Insertable)]
                    #[table_name="subtasks"]
                    struct NewSubtask {
                        task_id: i64,
                        active: bool,
                        number: i64,
                    }
                    diesel::insert_into(subtasks::table)
                        .values(&NewSubtask {
                            task_id: self.id,
                            active: true,
                            number: number,
                        }).execute(conn)?;
                    self.subtask(conn, number)
                },
                _ => Err(e),
            })
        })
    }

    pub fn subtask(&self, conn: &SqliteConnection, number: i64) -> Result<Subtask, diesel::result::Error> {
        use schema::subtasks::dsl;
        dsl::subtasks
            .filter(dsl::task_id.eq(self.id))
            .filter(dsl::number.eq(number))
            .get_result::<Subtask>(conn)
    }

    pub fn current(conn: &SqliteConnection) -> Option<Self> {
        use schema::tasks::dsl;
        current_stretch_scope(
            dsl::tasks.inner_join(
                schema::subtasks::dsl::subtasks
                .inner_join(schema::stretches::dsl::stretches)
            )).order(schema::stretches::dsl::start.desc())
            .select(schema::tasks::all_columns)
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
    pub fn for_code(conn: &SqliteConnection, code: &str) -> Result<(Project,Task,Subtask), DbOrMiscError> {
        let spec: SubtaskSpec = code.parse()?;
        SqliteConnection::transaction(conn, || {
            let project = get_project(conn, spec.project_code.as_ref())?;
            let task = project.task(conn, spec.task_number)?;
            let subtask = match spec.subtask_number {
                Some(number) => task.load_or_create_subtask(conn, number),
                None => task.latest_subtask(conn),
            }?;
            Ok((project,task,subtask))
        })
    }

    pub fn begin(&self, conn: &SqliteConnection) -> Result<(), DbOrMiscError> {
        use schema::stretches;
        #[derive(Insertable)]
        #[table_name="stretches"]
        struct NewStretch {
            subtask_id: i64,
            start: i64,
        }
        diesel::insert_into(stretches::table)
            .values(&NewStretch {
                subtask_id: self.id,
                start: SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs() as i64
            })
            .execute(conn)
            .map(|_| ())
            .map_err(std::convert::From::from)
    }
}

impl Stretch {
    pub fn current(conn: &SqliteConnection) -> Option<Self> {
        use schema::stretches::dsl;
        current_stretch_scope(dsl::stretches)
            .get_result::<Stretch>(conn)
            .ok()
    }

    pub fn stop_all(conn: &SqliteConnection) -> Result<(), DbOrMiscError> {
        Self::stop_all_at(conn, None)
    }

    pub fn stop_all_at(conn: &SqliteConnection, when: Option<String>) -> Result<(), DbOrMiscError> {
        use schema::stretches::dsl;
        let timestamp = match when {
            None => Ok(SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs() as i64),
            Some(timestring) => Auckland.datetime_from_str(timestring.as_ref(), "%Y-%m%dT%H:%M:%S")
                .map(|time| time.timestamp())
                .map_err(|e| e.to_string())
        }?;
        diesel::update(current_stretch_scope(dsl::stretches))
            .set(dsl::end.eq(timestamp))
            .execute(conn)
            .map(|_| ())
            .map_err(|e| DbOrMiscError::from(e))
    }

   pub fn time_in_range(&self, from: DateTime<Tz>, until: DateTime<Tz>) -> Option<Duration> {
      let from = *[from, self.start].iter().max().unwrap();
      let until = *[until, self.end?].iter().min().unwrap();
      if from > until {
          None
      } else {
          Some(until - from)
      }
    }

    pub fn dates(&self) -> impl Iterator<Item=NaiveDate> {
        enum I {
            R(NaiveDate,NaiveDate),
            N,
        }
        impl Iterator for I {
            type Item = NaiveDate;
            fn next(&mut self) -> Option<Self::Item> {
                match self {
                    Self::N => None,
                    Self::R(b,e) => {
                        let r = *b;
                        *self = if b == e {
                            Self::N
                        } else {
                            Self::R(b.succ(), *e)
                        };
                        Some(r)
                    },
                }
            }
        }
        match self.end {
            None => I::N,
            Some(e) => I::R(self.start.date().naive_local(), e.date().naive_local())
        }
    }
}

pub fn dates_to_timestamp_ranges<I: IntoIterator<Item=NaiveDate>>(source: I) -> impl Iterator<Item=core::ops::Range<DateTime<Tz>>> {
    source.into_iter().map(|d| {
        let until = Auckland.from_local_datetime(&d.succ().and_hms(0,0,0))
            .earliest()
            .unwrap();
        let from = Auckland.from_local_datetime(&d.and_hms(0,0,0))
            .latest()
            .unwrap();
        from .. until
    })
}

pub fn today() -> NaiveDate {
    Auckland
        .from_utc_datetime(&chrono::offset::Utc::now().naive_utc())
        .date()
        .naive_local()
}

pub fn time_since(conn: &SqliteConnection, from: NaiveDate) -> Result<HashMap<NaiveDate, HashMap<String, Duration>>, DbOrMiscError> {
    let mut result = HashMap::new();
    let today = today();
    for (project, task, subtask, stretch) in filter_stretch_date(schema::projects::dsl::projects
        .inner_join(
            schema::tasks::dsl::tasks
            .inner_join(
            schema::subtasks::dsl::subtasks
            .inner_join(
            schema::stretches::dsl::stretches
            ))
        ), from, today)
        .select((
                schema::projects::all_columns,
                schema::tasks::all_columns,
                schema::subtasks::all_columns,
                schema::stretches::all_columns
        )).load::<(Project,Task,Subtask,Stretch)>(conn)? {
        let code = format!("{}-{}-{}", project.code, task.number, subtask.number);
        for date in stretch.dates() {
            if date >= from && date <= today {
                let morning = Auckland.from_local_datetime(&date.and_hms(0,0,0)).earliest().unwrap();
                let night = Auckland.from_local_datetime(&date.succ().and_hms(0,0,0)).latest().unwrap();
                let duration = stretch.time_in_range(morning, night).unwrap();
                result.entry(date)
                    .or_insert_with(HashMap::new)
                    .entry(code.clone())
                    .and_modify(|d| {
                        *d = *d + duration
                    })
                    .or_insert(duration);
            }
        }
    }
    Ok(result)
}

fn current_stretch_scope<'a, S: diesel::query_dsl::methods::FilterDsl<diesel::expression::operators::IsNull<schema::stretches::columns::end>>>(scope: S) -> <S as diesel::query_dsl::filter_dsl::FilterDsl<diesel::expression::operators::IsNull<schema::stretches::columns::end>>>::Output {
    use super::schema::stretches::dsl;
    scope.filter(dsl::end.is_null())
}

fn filter_stretch_date<S: diesel::query_dsl::methods::FilterDsl<diesel::expression::operators::Eq<diesel::expression::grouped::Grouped<diesel::expression::operators::Or<diesel::expression::operators::Gt<schema::stretches::columns::start, diesel::expression::bound::Bound<diesel::sql_types::BigInt, i64>>, diesel::expression::operators::Lt<schema::stretches::columns::end, diesel::expression::bound::Bound<diesel::sql_types::Nullable<diesel::sql_types::BigInt>, i64>>>>, diesel::expression::bound::Bound<diesel::sql_types::Bool, bool>>>>(scope: S, from: chrono::naive::NaiveDate, until: chrono::naive::NaiveDate) -> <S as diesel::query_dsl::filter_dsl::FilterDsl<diesel::expression::operators::Eq<diesel::expression::grouped::Grouped<diesel::expression::operators::Or<diesel::expression::operators::Gt<schema::stretches::columns::start, diesel::expression::bound::Bound<diesel::sql_types::BigInt, i64>>, diesel::expression::operators::Lt<schema::stretches::columns::end, diesel::expression::bound::Bound<diesel::sql_types::Nullable<diesel::sql_types::BigInt>, i64>>>>, diesel::expression::bound::Bound<diesel::sql_types::Bool, bool>>>>::Output
 {
    use super::schema::stretches::dsl;
    let from = Auckland.from_local_datetime(&from.and_time(chrono::naive::NaiveTime::from_hms(0,0,0))).earliest().unwrap();
    let until = Auckland.from_local_datetime(&until.succ().and_time(chrono::naive::NaiveTime::from_hms(0,0,0))).latest().unwrap();
    scope.filter(dsl::start.gt(until.timestamp()).or(dsl::end.lt(from.timestamp())).eq(false))
}

#[derive(Debug)]
pub enum DbOrMiscError {
    Db(diesel::result::Error),
    Str(String),
}

impl std::convert::From<diesel::result::Error> for DbOrMiscError {
    fn from(err: diesel::result::Error) -> Self {
        Self::Db(err)
    }
}

impl std::convert::From<String> for DbOrMiscError {
    fn from(err: String) -> Self {
        Self::Str(err)
    }
}

impl std::convert::From<&str> for DbOrMiscError {
    fn from(err: &str) -> Self {
        <Self as std::convert::From<String>>::from(String::from(err))
    }
}

impl std::fmt::Display for DbOrMiscError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Db(err) => std::fmt::Display::fmt(err, f),
            Self::Str(err) => std::fmt::Display::fmt(err, f),
        }
    }
}
