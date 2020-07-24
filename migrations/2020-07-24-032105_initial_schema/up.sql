CREATE TABLE projects(id INTEGER PRIMARY KEY, code TEXT, directory TEXT, name TEXT);
CREATE TABLE tasks(id INTEGER PRIMARY KEY, project_id INTEGER, number INTEGER, active_subtask INTEGER, FOREIGN KEY(project_id) REFERENCES projects(id) ON DELETE CASCADE);
CREATE TABLE subtasks(id INTEGER PRIMARY KEY, task_id INTEGER, branch TEXT, description TEXT, active INTEGER, FOREIGN KEY(task_id) REFERENCES tasks(id) ON DELETE CASCADE);
CREATE TABLE stretches(id INTEGER PRIMARY KEY, subtask_id INTEGER, start INTEGER, end INTEGER, FOREIGN KEY(subtask_id) REFERENCES subtasks(id));