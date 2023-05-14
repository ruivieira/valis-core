use std::fmt::Error;

use rusqlite::{Connection, Row};

pub mod serializers;

pub trait DatabaseOperations<T> {
    fn save(&self, db: &str) -> Result<(), Error>;
    fn get(id: T, db: &str) -> Result<Self, rusqlite::Error> where Self: Sized;
    fn get_all(db: &str) -> Result<Vec<Self>, rusqlite::Error>
        where
            Self: Sized;
    fn map(row: &Row<'_>) -> Result<Self, rusqlite::Error> where Self: Sized;
}

fn get_sql_schema() -> Vec<String> {
    include_str!("tables.sql").split("---").map(|s| s.to_string()).collect::<Vec<String>>()
}

pub fn init_db(db: &str) -> Result<(), rusqlite::Error> {
    let conn = Connection::open(db)?;

    get_sql_schema().into_iter().for_each(|sql| {
        conn.execute(&sql, []).ok().unwrap();
    });

    Ok(())
}

pub fn get_connection(db: &str) -> Connection {
    match Connection::open(db) {
        Ok(conn) => conn,
        Err(e) => {
            println!("Failed to open database: {}", e);
            std::process::exit(1);
        }
    }
}