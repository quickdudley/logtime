table! {
    projects (id) {
        id -> Integer,
        code -> Text,
        directory -> Nullable<Text>,
        name -> Nullable<Text>,
    }
}

table! {
    stretches (id) {
        id -> Integer,
        subtask_id -> Integer,
        start -> Integer,
        end -> Nullable<Integer>,
    }
}

table! {
    subtasks (id) {
        id -> Integer,
        task_id -> Integer,
        branch -> Nullable<Text>,
        description -> Nullable<Text>,
        active -> Integer,
    }
}

table! {
    tasks (id) {
        id -> Integer,
        project_id -> Integer,
        number -> Integer,
        active_subtask -> Nullable<Integer>,
    }
}

joinable!(stretches -> subtasks (subtask_id));
joinable!(subtasks -> tasks (task_id));
joinable!(tasks -> projects (project_id));

allow_tables_to_appear_in_same_query!(
    projects,
    stretches,
    subtasks,
    tasks,
);
