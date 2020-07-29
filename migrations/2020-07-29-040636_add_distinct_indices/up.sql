CREATE UNIQUE INDEX idx_project_code ON projects(code);
CREATE UNIQUE INDEX idx_task_number ON tasks(project_id, number);