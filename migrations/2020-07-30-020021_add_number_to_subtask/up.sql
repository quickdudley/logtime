ALTER TABLE subtasks ADD COLUMN number INTEGER NOT NULL DEFAULT 1;
UPDATE subtasks SET number = (SELECT count(1) + 1 FROM subtasks AS other WHERE other.task_id = subtasks.task_id AND other.id < subtasks.id);
CREATE UNIQUE INDEX idx_subtask_number ON subtasks(task_id, number);