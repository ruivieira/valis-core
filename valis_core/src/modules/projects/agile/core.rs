use std::fmt::Error;

use chrono::{DateTime, Utc};
use rusqlite::{params, Connection, Result, Row};
use serde::{Deserialize, Serialize};
use termion::{color, style};
use uuid::Uuid;

use db::DatabaseOperations;

use crate::modules::db;
use crate::modules::db::get_connection;
use crate::modules::db::serializers::SerializableDateTime;
use crate::modules::tasks::todoist::core::Task as TodoisTask;

#[derive(Serialize, Deserialize)]
pub struct Project {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub created_at: SerializableDateTime,
    pub updated_at: SerializableDateTime,
}

impl DatabaseOperations<String> for Project {
    /// Create a new project and save it
    fn save(&self, db: &str) -> Result<(), Error> {
        let conn = Connection::open(db).ok().unwrap();
        let id = Uuid::new_v4();
        let now = Utc::now().to_string();

        conn.execute(
            "INSERT INTO project (id, name, description, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![id.to_string(), self.name, self.description, now, now],
        ).ok().unwrap();

        Ok(())
    }

    fn get(_id: String, _db: &str) -> Result<Self, rusqlite::Error>
    where
        Self: Sized,
    {
        todo!()
    }

    // List All projects
    fn get_all(db: &str) -> Result<Vec<Project>, rusqlite::Error> {
        let conn = Connection::open(db).ok().unwrap();
        let mut stmt = conn.prepare("SELECT * FROM project").ok().unwrap();
        let rows = stmt
            .query_map((), |row| {
                let id: Vec<u8> = row.get(0).ok().unwrap();
                Ok(Project {
                    id: Uuid::from_slice(&id).unwrap(),
                    name: row.get(1).ok().unwrap(),
                    description: row.get(2).ok().unwrap(),
                    created_at: row.get(3).ok().unwrap(),
                    updated_at: row.get(4).ok().unwrap(),
                })
            })
            .ok()
            .unwrap();

        let mut projects = Vec::new();
        for project in rows {
            projects.push(project.ok().unwrap());
        }

        Ok(projects)
    }

    fn map(row: &Row<'_>) -> std::result::Result<Self, rusqlite::Error>
    where
        Self: Sized,
    {
        todo!()
    }
}

impl Default for Project {
    fn default() -> Self {
        let now = SerializableDateTime::now();
        Self {
            id: Uuid::new_v4(),
            name: "Default name".to_string(),
            description: "Default description".to_string(),
            created_at: now.clone(),
            updated_at: now.clone(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Sprint {
    pub id: Uuid,
    pub project_id: Uuid,
    pub name: String,
    pub start_date: SerializableDateTime,
    pub end_date: SerializableDateTime,
    pub created_at: SerializableDateTime,
    pub updated_at: SerializableDateTime,
}

impl DatabaseOperations<String> for Sprint {
    /// Save a Sprint
    fn save(&self, db: &str) -> Result<(), Error> {
        let conn = Connection::open(db).ok().unwrap();

        let now = Utc::now().to_rfc3339().to_string();

        conn.execute(
            "INSERT INTO sprint (id, project_id, name, start_date, end_date, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![self.id.to_string(), &self.project_id.to_string(), &self.name, &self.start_date.with_timezone(&Utc).to_string(), &self.end_date.with_timezone(&Utc).to_string(), now, now],
        ).ok().unwrap();

        Ok(())
    }

    fn get(id: String, db: &str) -> Result<Self, rusqlite::Error>
    where
        Self: Sized,
    {
        let conn = get_connection(db);
        let mut query = conn.prepare("SELECT * FROM sprint WHERE id = ?")?;

        query.query_row(params![&id], Sprint::map)
    }

    /// List all sprints
    fn get_all(db: &str) -> Result<Vec<Sprint>, rusqlite::Error> {
        let conn = Connection::open(db)?;
        let mut stmt = conn.prepare("SELECT * FROM sprint")?;
        let rows = stmt.query_map((), Sprint::map)?;

        let mut sprints = Vec::new();
        for sprint_res in rows {
            sprints.push(sprint_res?);
        }

        Ok(sprints)
    }
    fn map(row: &Row<'_>) -> Result<Self, rusqlite::Error>
    where
        Self: Sized,
    {
        let id: String = row.get(0)?;
        let project_id: String = row.get(1)?;
        let start_date: String = row.get(3)?;
        let end_date: String = row.get(4)?;
        let created_at: String = row.get(5)?;
        let updated_at: String = row.get(6)?;
        Ok(Sprint {
            id: Uuid::parse_str(&id).unwrap(),
            project_id: Uuid::parse_str(&project_id).unwrap(),
            name: row.get(2)?,
            start_date: SerializableDateTime::parse_from_rfc3339(&start_date)
                .unwrap()
                .with_timezone(&Utc),
            end_date: SerializableDateTime::parse_from_rfc3339(&end_date)
                .unwrap()
                .with_timezone(&Utc),
            created_at: SerializableDateTime::parse_from_rfc3339(&created_at)
                .unwrap()
                .with_timezone(&Utc),
            updated_at: SerializableDateTime::parse_from_rfc3339(&updated_at)
                .unwrap()
                .with_timezone(&Utc),
        })
    }
}

pub fn sprint_get_all(db: &str) -> Result<Vec<Sprint>, rusqlite::Error> {
    Sprint::get_all(db)
}

impl Default for Sprint {
    fn default() -> Self {
        let now = SerializableDateTime::now();
        Self {
            id: Uuid::new_v4(),
            project_id: Uuid::new_v4(),
            name: "Default name".to_string(),
            start_date: now.clone(),
            end_date: now.add_weeks(3),
            created_at: now.clone(),
            updated_at: now.clone(),
        }
    }
}

impl Sprint {
    pub fn is_active(&self) -> bool {
        let now = Utc::now();
        self.start_date.get_utc() <= now && now <= self.end_date.get_utc()
    }
}

// Delete a project, given the project id
pub fn delete_project_by_id(conn: &Connection, id: Uuid) -> Result<()> {
    conn.execute("DELETE FROM project WHERE id = ?1", params![id.to_string()])?;

    Ok(())
}

// Delete a project, given the project name
pub fn delete_project_by_name(conn: &Connection, name: &str) -> Result<()> {
    conn.execute("DELETE FROM project WHERE name = ?1", params![name])?;

    Ok(())
}

// Create a Sprint
pub fn create_sprint(
    conn: &Connection,
    project_id: Uuid,
    name: &str,
    start_date: DateTime<Utc>,
    end_date: DateTime<Utc>,
) -> Result<()> {
    let id = Uuid::new_v4();
    let now = Utc::now().to_string();

    conn.execute(
        "INSERT INTO sprint (id, project_id, name, start_date, end_date, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![id.to_string(), project_id.to_string(), name, start_date.to_string(), end_date.to_string(), now, now],
    )?;

    Ok(())
}

// Delete a Sprint
pub fn delete_sprint(conn: &Connection, id: Uuid) -> Result<()> {
    conn.execute("DELETE FROM sprint WHERE id = ?1", params![id.to_string()])?;

    Ok(())
}

// List all sprints for a project
pub fn list_sprints_for_project(conn: &Connection, project_id: Uuid) -> Result<Vec<Sprint>> {
    let mut stmt = conn.prepare("SELECT * FROM sprint WHERE project_id = ?1")?;
    let rows = stmt.query_map(params![project_id.to_string()], Sprint::map)?;

    let mut sprints = Vec::new();
    for sprint_res in rows {
        sprints.push(sprint_res?);
    }

    Ok(sprints)
}

pub fn print_sprint_info(db: &str, sprint_id: Uuid) -> Result<()> {
    let conn = get_connection(db);
    let sprint_info = Sprint::get(sprint_id.to_string(), db).ok().unwrap();

    println!(
        "\n{}{}Sprint Information{}",
        style::Bold,
        color::Fg(color::Blue),
        style::Reset
    );
    println!(
        "{}ID:{} {}",
        color::Fg(color::Green),
        style::Reset,
        sprint_info.id.to_string()
    );
    println!(
        "{}Name:{} {}",
        color::Fg(color::Green),
        style::Reset,
        sprint_info.name
    );
    println!(
        "{}Start Date:{} {}",
        color::Fg(color::Green),
        style::Reset,
        sprint_info.start_date.get_utc().to_string()
    );
    println!(
        "{}End Date:{} {}",
        color::Fg(color::Green),
        style::Reset,
        sprint_info.end_date.get_utc().to_string()
    );

    let now = Utc::now();
    let days_to_finish = sprint_info
        .end_date
        .get_utc()
        .signed_duration_since(now)
        .num_days();
    println!(
        "{}Days to Sprint Finish:{} {}",
        color::Fg(color::Green),
        style::Reset,
        days_to_finish.to_string()
    );

    let mut task_query = conn.prepare(
        "
        SELECT task.* FROM task
        INNER JOIN sprint_todoist_task ON task.id = sprint_todoist_task.todoist_task_id
        WHERE sprint_todoist_task.sprint_id = ?
    ",
    )?;
    let task_rows = task_query.query_map(params![sprint_id.to_string()], |row| {
        Ok(TodoisTask {
            id: row.get(0)?,
            content: row.get(1)?,
            // TODO: Fetch all the labels
            labels: vec![],
        })
    })?;

    let tasks: Result<Vec<TodoisTask>, _> = task_rows.collect();
    match tasks {
        Ok(tasks) => {
            println!(
                "\n{}{}Tasks{}",
                style::Bold,
                color::Fg(color::Blue),
                style::Reset
            );
            for task in tasks {
                println!(
                    "{}Task ID:{} {}",
                    color::Fg(color::Green),
                    style::Reset,
                    task.id
                );
                println!(
                    "{}Content:{} {}",
                    color::Fg(color::Green),
                    style::Reset,
                    task.content.unwrap_or_else(|| "None".to_string())
                );
                println!(
                    "{}Labels:{} {:?}",
                    color::Fg(color::Green),
                    style::Reset,
                    task.labels
                );
            }
        }
        Err(e) => println!("Failed to get tasks: {}", e),
    }

    Ok(())
}
