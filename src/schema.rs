table! {
    projects (id) {
        id -> BigInt,
        code -> Text,
        directory -> Nullable<Text>,
        name -> Nullable<Text>,
    }
}

table! {
    stretches (id) {
        id -> BigInt,
        subtask_id -> BigInt,
        start -> BigInt,
        end -> Nullable<BigInt>,
    }
}

table! {
    subtasks (id) {
        id -> BigInt,
        task_id -> BigInt,
        branch -> Nullable<Text>,
        description -> Nullable<Text>,
        active -> BigInt,
    }
}

table! {
    tasks (id) {
        id -> BigInt,
        project_id -> BigInt,
        number -> BigInt,
        active_subtask -> Nullable<BigInt>,
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
