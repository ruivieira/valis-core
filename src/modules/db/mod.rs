use std::fmt::Error;

use rusqlite::Connection;

pub mod serializers;

pub trait DatabaseOperations<T> {
    fn save(&self, db: &str) -> Result<(), Error>;
    // fn get(&self, id: T, db: &str) -> Self where Self: Sized;
    fn get_all(db: &str) -> Result<Vec<Self>, Error>
        where
            Self: Sized;
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
