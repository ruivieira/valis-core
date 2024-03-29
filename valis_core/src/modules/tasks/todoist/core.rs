use std::result::Result;

use rusqlite::{params, Connection, Row};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use db::DatabaseOperations;

use crate::modules::db;
use crate::modules::db::get_connection;
use crate::modules::projects::agile::core::Sprint;

#[derive(Debug, Deserialize, Serialize)]
pub struct Task {
    pub id: String,
    pub content: Option<String>,
    pub labels: Vec<String>,
    // add other fields as necessary
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SprintTask {
    pub sprint_id: Uuid,
    pub todoist_task_id: String,
}

impl Task {
    pub fn add_to_sprint(&self, db: &str, sprint: Sprint) -> Result<(), rusqlite::Error> {
        add_task_to_sprint(db, &sprint.id, self.id.to_string())
    }
}

impl DatabaseOperations<String> for Task {
    fn save(&self, db: &str) -> Result<(), std::fmt::Error> {
        todo!()
    }

    fn get(id: String, db: &str) -> Result<Self, rusqlite::Error>
    where
        Self: Sized,
    {
        todo!()
    }

    fn get_all(db: &str) -> Result<Vec<Task>, rusqlite::Error> {
        let conn = get_connection(db);

        let mut stmt = conn.prepare("SELECT * FROM todoist_tasks").ok().unwrap();
        let tasks_iter = stmt
            .query_map([], |row| {
                Ok(Task {
                    id: row.get(0).ok().unwrap(),
                    content: row.get(1).ok().unwrap(),
                    labels: vec![], // We'll get the tags later
                })
            })
            .ok()
            .unwrap();

        let mut tasks = Vec::new();
        for task_result in tasks_iter {
            let mut task = task_result.ok().unwrap();
            let mut stmt = conn
                .prepare(
                    "
            SELECT todoist_labels.label FROM todoist_labels
            JOIN todoist_task_labels ON todoist_labels.id = todoist_task_labels.todoist_label_id
            WHERE todoist_task_labels.todoist_task_id = ?
        ",
                )
                .ok()
                .unwrap();
            let labels_iter = stmt
                .query_map(params![task.id], |row| row.get(0))
                .ok()
                .unwrap();

            for label_result in labels_iter {
                task.labels.push(label_result.ok().unwrap());
            }

            tasks.push(task);
        }

        Ok(tasks)
    }

    fn map(row: &Row<'_>) -> Result<Self, rusqlite::Error>
    where
        Self: Sized,
    {
        todo!()
    }
}

pub fn get_task_by_id(db: &str, id: String) -> Result<Task, std::fmt::Error> {
    let conn = get_connection(db);

    let mut stmt = conn
        .prepare("SELECT * FROM tasks WHERE id = ?")
        .ok()
        .unwrap();
    let mut task = stmt
        .query_row(params![id], |row| {
            Ok(Task {
                id: row.get(0).ok().unwrap(),
                content: row.get(1).ok().unwrap(),
                labels: vec![], // We'll get the tags later
            })
        })
        .ok()
        .unwrap();

    let mut stmt = conn
        .prepare(
            "
        SELECT todoist_labels.label FROM todoist_labels
        JOIN todoist_task_labels ON todoist_labels.id = todoist_task_labels.label_id
        WHERE todoist_task_labels.task_id = ?
    ",
        )
        .ok()
        .unwrap();
    let labels_iter = stmt.query_map(params![id], |row| row.get(0)).ok().unwrap();

    for label_result in labels_iter {
        task.labels.push(label_result.ok().unwrap());
    }

    Ok(task)
}

pub fn get_all_tasks(db: &str) -> Result<Vec<Task>, rusqlite::Error> {
    let conn = get_connection(db);

    let mut stmt = conn.prepare("SELECT * FROM todoist_tasks").ok().unwrap();
    let tasks_iter = stmt
        .query_map([], |row| {
            Ok(Task {
                id: row.get(0).ok().unwrap(),
                content: row.get(1).ok().unwrap(),
                labels: vec![], // We'll get the tags later
            })
        })
        .ok()
        .unwrap();

    let mut tasks = Vec::new();
    for task_result in tasks_iter {
        let mut task = task_result.ok().unwrap();
        let mut stmt = conn
            .prepare(
                "
            SELECT todoist_labels.label FROM todoist_labels
            JOIN todoist_task_labels ON todoist_labels.id = todoist_task_labels.todoist_label_id
            WHERE todoist_task_labels.todoist_task_id = ?
        ",
            )
            .ok()
            .unwrap();
        let labels_iter = stmt
            .query_map(params![task.id], |row| row.get(0))
            .ok()
            .unwrap();

        for label_result in labels_iter {
            task.labels.push(label_result.ok().unwrap());
        }

        tasks.push(task);
    }

    Ok(tasks)
}

async fn get_todoist_tasks(todoist_token: &str) -> Result<Vec<Task>, reqwest::Error> {
    let client = reqwest::Client::new();
    let response = client
        .get("https://api.todoist.com/rest/v2/tasks")
        .header("Authorization", format!("Bearer {}", todoist_token))
        .send()
        .await?
        .text()
        .await?;

    Ok(serde_json::from_str::<Vec<Task>>(&response).ok().unwrap())
}

fn sync_to_db(tasks: &Vec<Task>, db: &str) -> rusqlite::Result<()> {
    let conn = Connection::open(db)?;

    for task in tasks {
        let result: Result<String, rusqlite::Error> = conn.query_row(
            "SELECT id FROM todoist_tasks WHERE id = ?1",
            params![task.id],
            |row| row.get(0),
        );

        match result {
            Ok(_) => {
                // task exists in the database, update it
                conn.execute(
                    "UPDATE todoist_tasks SET content = ?1 WHERE id = ?2",
                    params![task.content, task.id],
                )?;
            }
            Err(_) => {
                // task doesn't exist in the database, insert it
                conn.execute(
                    "INSERT INTO todoist_tasks (id, content) VALUES (?1, ?2)",
                    params![task.id, task.content],
                )?;
            }
        }
        for label in &task.labels {
            // Insert tag if it doesn't exist
            conn.execute(
                "INSERT OR IGNORE INTO todoist_labels (label) VALUES (?)",
                [label],
            )?;

            // Get tag id
            let label_id: i32 = conn.query_row(
                "SELECT id FROM todoist_labels WHERE label = ?",
                [label],
                |row| row.get(0),
            )?;

            // Link task and tag
            conn.execute(
                "INSERT OR IGNORE INTO todoist_task_labels (todoist_task_id, todoist_label_id) VALUES (?, ?)",
                [&task.id, &(label_id.to_string())],
            )?;
        }
    }

    // delete tasks from database that are not in Todoist
    let db_tasks: Vec<String> = conn
        .prepare("SELECT id FROM todoist_tasks")?
        .query_map([], |row| row.get(0))?
        .collect::<Result<_, _>>()?;

    for db_task in db_tasks {
        if !tasks.iter().any(|task| task.id == db_task) {
            conn.execute("DELETE FROM todoist_tasks WHERE id = ?1", params![db_task])?;
        }
    }

    Ok(())
}

pub fn add_task_to_sprint(
    db: &str,
    sprint_id: &Uuid,
    task_id: String,
) -> Result<(), rusqlite::Error> {
    let conn = get_connection(db);
    conn.execute(
        "INSERT INTO sprint_todoist_task (sprint_id, todoist_task_id) VALUES (?1, ?2)",
        [&sprint_id.to_string(), &task_id],
    )?;

    Ok(())
}

pub async fn sync(token: &str, db: &str) -> Result<(), std::fmt::Error> {
    let tasks = get_todoist_tasks(token).await.ok().unwrap();
    sync_to_db(&tasks, db).ok().unwrap();

    Ok(())
}
