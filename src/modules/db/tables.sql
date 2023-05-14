CREATE TABLE IF NOT EXISTS todoist_tasks
(
    id      VARCHAR PRIMARY KEY,
    content TEXT NOT NULL
);
---
CREATE TABLE IF NOT EXISTS todoist_labels
(
    id    INTEGER PRIMARY KEY,
    label TEXT NOT NULL UNIQUE
);
---
CREATE TABLE IF NOT EXISTS todoist_task_labels
(
    todoist_task_id  VARCHAR,
    todoist_label_id INTEGER,
    PRIMARY KEY (todoist_task_id, todoist_label_id),
    FOREIGN KEY (todoist_task_id) REFERENCES todoist_tasks (id) ON DELETE CASCADE,
    FOREIGN KEY (todoist_label_id) REFERENCES todoist_labels (id) ON DELETE CASCADE
);
---
CREATE TABLE IF NOT EXISTS sprint_todoist_task
(
    sprint_id       TEXT NOT NULL,
    todoist_task_id TEXT NOT NULL,
    PRIMARY KEY (sprint_id, todoist_task_id)
);
---
CREATE TABLE IF NOT EXISTS project
(
    id          BLOB PRIMARY KEY,
    name        TEXT NOT NULL,
    description TEXT NOT NULL,
    created_at  TEXT NOT NULL,
    updated_at  TEXT NOT NULL
);
---
CREATE TABLE IF NOT EXISTS sprint
(
    id         BLOB PRIMARY KEY,
    project_id BLOB NOT NULL,
    name       TEXT NOT NULL,
    start_date TEXT NOT NULL,
    end_date   TEXT NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    FOREIGN KEY (project_id) REFERENCES project (id)
);