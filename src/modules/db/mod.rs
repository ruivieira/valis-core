use std::fmt::Error;

pub mod serializers;

pub trait DatabaseOperations<T> {
    fn save(&self, db: &str) -> Result<(), Error>;
    // fn get(&self, id: T, db: &str) -> Self where Self: Sized;
    fn get_all(db: &str) -> Result<Vec<Self>, Error>
    where
        Self: Sized;
}
