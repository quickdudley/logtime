DROP INDEX idx_subtask_number;
CREATE TABLE subtasks_backup(id INTEGER NOT NULL PRIMARY KEY, task_id INTEGER NOT NULL, branch TEXT, description TEXT, active INTEGER NOT NULL DEFAULT 0, FOREIGN KEY(task_id) REFERENCES tasks(id) ON DELETE CASCADE);
INSERT INTO subtasks_backup SELECT id, task_id, branch, description, active FROM subtasks;
DROP TABLE subtasks;
ALTER TABLE subtasks_backup RENAME TO subtasks;