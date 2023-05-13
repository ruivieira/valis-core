use std::fmt::Error;

use chrono::{DateTime, Utc};
use rusqlite::{Connection, params, Result};
use serde::{Deserialize, Serialize, Serializer};

use uuid::Uuid;

use crate::modules::db;
use crate::modules::db::serializers;
use crate::modules::db::serializers::SerializableDateTime;

#[derive(Serialize, Deserialize)]
pub struct Project {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub created_at: serializers::SerializableDateTime,
    pub updated_at: serializers::SerializableDateTime,
}

impl db::DatabaseOperations<String> for Project {
    /// Create a new project and save it
    fn save(&self, db: &str) -> Result<(), Error> {
        let conn = Connection::open(db).ok().unwrap();
        let id = Uuid::new_v4();
        let now = Utc::now().to_string();

        conn.execute(
            "INSERT INTO project (id, name, description, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![id.as_bytes(), self.name, self.description, now, now],
        ).ok().unwrap();

        Ok(())
    }
    // List All projects
    fn get_all(&self, db: &str) -> Result<Vec<Project>, Error> {
        let conn = Connection::open(db).ok().unwrap();
        let mut stmt = conn.prepare("SELECT * FROM project").ok().unwrap();
        let rows = stmt.query_map((), |row| {
            let id: Vec<u8> = row.get(0).ok().unwrap();
            Ok(Project {
                id: Uuid::from_slice(&id).unwrap(),
                name: row.get(1).ok().unwrap(),
                description: row.get(2).ok().unwrap(),
                created_at: row.get(3).ok().unwrap(),
                updated_at: row.get(4).ok().unwrap(),
            })
        }).ok().unwrap();

        let mut projects = Vec::new();
        for project in rows {
            projects.push(project.ok().unwrap());
        }

        Ok(projects)
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

#[derive(Serialize, Deserialize)]
pub struct Sprint {
    pub id: Uuid,
    pub project_id: Uuid,
    pub name: String,
    pub start_date: serializers::SerializableDateTime,
    pub end_date: serializers::SerializableDateTime,
    pub created_at: serializers::SerializableDateTime,
    pub updated_at: serializers::SerializableDateTime,
}

pub fn init_db(db: &str) -> Result<()> {
    let conn = Connection::open(db)?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS project (
             id BLOB PRIMARY KEY,
             name TEXT NOT NULL,
             description TEXT NOT NULL,
             created_at TEXT NOT NULL,
             updated_at TEXT NOT NULL
         )",
        params![],
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS sprint (
             id BLOB PRIMARY KEY,
             project_id BLOB NOT NULL,
             name TEXT NOT NULL,
             start_date TEXT NOT NULL,
             end_date TEXT NOT NULL,
             created_at TEXT NOT NULL,
             updated_at TEXT NOT NULL,
             FOREIGN KEY(project_id) REFERENCES project(id)
         )",
        params![],
    )?;

    Ok(())
}


// Delete a project, given the project id
pub fn delete_project_by_id(conn: &Connection, id: Uuid) -> Result<()> {
    conn.execute(
        "DELETE FROM project WHERE id = ?1",
        params![id.as_bytes()],
    )?;

    Ok(())
}

// Delete a project, given the project name
pub fn delete_project_by_name(conn: &Connection, name: &str) -> Result<()> {
    conn.execute(
        "DELETE FROM project WHERE name = ?1",
        params![name],
    )?;

    Ok(())
}

// Create a Sprint
pub fn create_sprint(conn: &Connection, project_id: Uuid, name: &str, start_date: DateTime<Utc>, end_date: DateTime<Utc>) -> Result<()> {
    let id = Uuid::new_v4();
    let now = Utc::now().to_string();

    conn.execute(
        "INSERT INTO sprint (id, project_id, name, start_date, end_date, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![id.as_bytes(), project_id.as_bytes(), name, start_date.to_string(), end_date.to_string(), now, now],
    )?;

    Ok(())
}

// Delete a Sprint
pub fn delete_sprint(conn: &Connection, id: Uuid) -> Result<()> {
    conn.execute(
        "DELETE FROM sprint WHERE id = ?1",
        params![id.as_bytes()],
    )?;

    Ok(())
}


// List all sprints for a project
pub fn list_sprints_for_project(conn: &Connection, project_id: Uuid) -> Result<Vec<Sprint>> {
    let mut stmt = conn.prepare("SELECT * FROM sprint WHERE project_id = ?1")?;
    let rows = stmt.query_map(params![project_id.as_bytes()], |row| {
        let id: Vec<u8> = row.get(0)?;
        let project_id: Vec<u8> = row.get(1)?;
        let start_date: String = row.get(3)?;
        let end_date: String = row.get(4)?;
        let created_at: String = row.get(5)?;
        let updated_at: String = row.get(6)?;
        Ok(Sprint {
            id: Uuid::from_slice(&id).unwrap(),
            project_id: Uuid::from_slice(&project_id).unwrap(),
            name: row.get(2)?,
            start_date: serializers::SerializableDateTime::parse_from_rfc3339(&start_date).unwrap().with_timezone(&Utc),
            end_date: serializers::SerializableDateTime::parse_from_rfc3339(&end_date).unwrap().with_timezone(&Utc),
            created_at: serializers::SerializableDateTime::parse_from_rfc3339(&created_at).unwrap().with_timezone(&Utc),
            updated_at: serializers::SerializableDateTime::parse_from_rfc3339(&updated_at).unwrap().with_timezone(&Utc),
        })
    })?;

    let mut sprints = Vec::new();
    for sprint_res in rows {
        sprints.push(sprint_res?);
    }

    Ok(sprints)
}

// List all sprints
pub fn list_all_sprints(conn: &Connection) -> Result<Vec<Sprint>> {
    let mut stmt = conn.prepare("SELECT * FROM sprint")?;
    let rows = stmt.query_map((), |row| {
        let id: Vec<u8> = row.get(0)?;
        let project_id: Vec<u8> = row.get(1)?;
        let start_date: String = row.get(3)?;
        let end_date: String = row.get(4)?;
        let created_at: String = row.get(5)?;
        let updated_at: String = row.get(6)?;
        Ok(Sprint {
            id: Uuid::from_slice(&id).unwrap(),
            project_id: Uuid::from_slice(&project_id).unwrap(),
            name: row.get(2)?,
            start_date: serializers::SerializableDateTime::parse_from_rfc3339(&start_date).unwrap().with_timezone(&Utc),
            end_date: serializers::SerializableDateTime::parse_from_rfc3339(&end_date).unwrap().with_timezone(&Utc),
            created_at: serializers::SerializableDateTime::parse_from_rfc3339(&created_at).unwrap().with_timezone(&Utc),
            updated_at: serializers::SerializableDateTime::parse_from_rfc3339(&updated_at).unwrap().with_timezone(&Utc),
        })
    })?;

    let mut sprints = Vec::new();
    for sprint_res in rows {
        sprints.push(sprint_res?);
    }

    Ok(sprints)
}
