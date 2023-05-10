use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;

use kdbx_rs;
use kdbx_rs::{CompositeKey, Database};

#[derive(Clone)]
pub struct Secret {
    pub entry: String,
    pub field: String,
    pub env_var: String,
}

pub fn read_fields_from_entry(
    kdbx_file_path: &PathBuf,
    password: String,
    secrets: Vec<Secret>,
) -> Result<Vec<Option<String>>, Box<dyn std::error::Error>> {
    // Open the KDBX file
    let file = File::open(kdbx_file_path).unwrap();
    let reader = BufReader::new(file);

    // Create a composite key using the provided password
    let composite_key = CompositeKey::from_password(&password);

    // Read the KDBX file and create a database
    // let db = kdbx_rs::(reader, &composite_key)?;
    let locked_db = kdbx_rs::open(kdbx_file_path).unwrap();
    let db = locked_db.unlock(&composite_key).ok().unwrap();

    let values = secrets
        .into_iter()
        .map(|secret| {
            let entry = db
                .root()
                .entries()
                .find(|e| e.title().unwrap() == secret.entry);

            match entry {
                None => return None,
                Some(e) => {
                    let field_value = e.fields().find(|f| f.key() == secret.field);
                    match field_value {
                        None => return None,
                        Some(f) => return Some(f.value().unwrap().to_string()),
                    }
                }
            };
        })
        .collect::<Vec<Option<String>>>();

    Ok(values)
}
