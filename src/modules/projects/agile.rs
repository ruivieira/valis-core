// use serde::{Deserialize, Serialize};
// use sqlx::{Pool, Sqlite};
// use sqlx::sqlite::SqlitePoolOptions;
//
// use chrono::{DateTime, Utc};
// use uuid::Uuid;
//
// #[derive(Serialize, Deserialize)]
// struct Project {
//     id: Uuid,
//     name: String,
//     description: String,
//     created_at: DateTime<Utc>,
//     updated_at: DateTime<Utc>,
// }
//
// #[derive(Serialize, Deserialize)]
// struct Sprint {
//     id: Uuid,
//     project_id: Uuid,
//     name: String,
//     start_date: DateTime<Utc>,
//     end_date: DateTime<Utc>,
//     created_at: DateTime<Utc>,
//     updated_at: DateTime<Utc>,
// }
//
// #[derive(Serialize, Deserialize)]
// struct Task {
//     id: Uuid,
//     sprint_id: Option<Uuid>,
//     project_id: Uuid,
//     title: String,
//     description: Option<String>,
//     status: String,
//     priority: String,
//     created_at: DateTime<Utc>,
//     updated_at: DateTime<Utc>,
// }
//
// #[derive(Serialize, Deserialize)]
// struct TaskComment {
//     id: Uuid,
//     task_id: Uuid,
//     content: String,
//     created_at: DateTime<Utc>,
//     updated_at: DateTime<Utc>,
// }
//
// async fn connect() -> Result<Pool<Sqlite>, Box<dyn std::error::Error>> {
//     let database_url = "sqlite:./my_database.sqlite3";
//     let pool = SqlitePoolOptions::new()
//         .max_connections(5)
//         .connect(database_url)
//         .await?;
//
//     // Use the pool to execute queries and interact with the database here
//
//     Ok(pool)
// }
