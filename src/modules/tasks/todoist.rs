use reqwest::Error;
use rusqlite::{Connection, params, Result};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Task {
    id: String,
    content: Option<String>,
    // add other fields as necessary
}

async fn get_todoist_tasks(todoist_token: &str) -> Result<Vec<Task>, Error> {
    let client = reqwest::Client::new();
    let response = client.get("https://api.todoist.com/rest/v2/tasks")
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
        let result: Result<String> = conn.query_row(
            "SELECT id FROM tasks WHERE id = ?1",
            params![task.id],
            |row| row.get(0),
        );

        match result {
            Ok(_) => {
                // task exists in the database, update it
                conn.execute(
                    "UPDATE tasks SET content = ?1 WHERE id = ?2",
                    params![task.content, task.id],
                )?;
            }
            Err(_) => {
                // task doesn't exist in the database, insert it
                conn.execute(
                    "INSERT INTO tasks (id, content) VALUES (?1, ?2)",
                    params![task.id, task.content],
                )?;
            }
        }
    }

    // delete tasks from database that are not in Todoist
    let db_tasks: Vec<String> = conn.prepare("SELECT id FROM tasks")?
        .query_map([], |row| row.get(0))?
        .collect::<Result<_, _>>()?;

    for db_task in db_tasks {
        if !tasks.iter().any(|task| task.id == db_task) {
            conn.execute(
                "DELETE FROM tasks WHERE id = ?1",
                params![db_task],
            )?;
        }
    }

    Ok(())
}

pub fn init_db(db: &str) -> rusqlite::Result<()> {
    let conn = Connection::open(db)?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS tasks (
            id TEXT PRIMARY KEY,
            content TEXT NOT NULL
        )",
        [],
    )?;

    Ok(())
}


pub async fn sync(token: &str, db: &str) -> Result<(), Box<dyn std::error::Error>> {
    let tasks = get_todoist_tasks(token).await?;
    sync_to_db(&tasks, db)?;

    Ok(())
}
