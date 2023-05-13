use std::error::Error;
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};

use rlua::{Context, Lua};
use serde::{Deserialize, Serialize};
use serde_yaml::Mapping;
use serde_yaml::Value as YamlValue;

fn update_yaml_value_in_place(
    doc: &mut YamlValue,
    path: &str,
    new_value: &str,
) -> Result<(), Box<dyn Error>> {
    let path_segments: Vec<&str> = path.split('.').collect();
    let mut current_node = doc;

    for segment in path_segments {
        match current_node {
            YamlValue::Mapping(ref mut map) => {
                if let Some(value) = map.get_mut(&YamlValue::String(segment.to_string())) {
                    current_node = value;
                } else {
                    return Err(format!("Invalid path segment: {}", segment).into());
                }
            }
            _ => return Err(format!("Invalid path segment: {}", segment).into()),
        }
    }

    *current_node = YamlValue::String(new_value.to_string());
    Ok(())
}

/// Update a YAML file with the provided key-value pairs.
/// # Arguments
/// * `file_path` - The path to the YAML file.
/// * `path` - The dot-separated path to the YAML value to update.
/// * `new_value` - The new value to set.
pub fn update_yaml_value(
    file_path: &str,
    path: &str,
    new_value: &str,
) -> Result<(), Box<dyn Error>> {
    // Read the YAML file
    let mut file = File::open(file_path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    // Parse the YAML content
    let mut doc: YamlValue = serde_yaml::from_str(&contents)?;

    // Update the YAML value in-place
    update_yaml_value_in_place(&mut doc, path, new_value)?;

    // Write the updated content back to the file
    let mut file = OpenOptions::new()
        .write(true)
        .truncate(true)
        .open(file_path)?;
    let updated_contents = serde_yaml::to_string(&doc)?;
    file.write_all(updated_contents.as_bytes())?;

    Ok(())
}

/// Get a YAML value from a file.
/// # Arguments
/// * `file_path` - The path to the YAML file.
/// * `yaml_key_path` - The dot-separated path to the YAML value to retrieve.
/// # Returns
/// The YAML value as a `String`.
pub fn get_yaml_value(
    file_path: &str,
    yaml_key_path: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    // Read the YAML file
    let mut file = File::open(file_path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    // Parse the YAML
    let yaml_data: YamlValue = serde_yaml::from_str(&contents)?;

    // Split the dot-separated key path
    let keys: Vec<&str> = yaml_key_path.split('.').collect();

    // Traverse and get the YAML value
    let mut current_node = &yaml_data;
    for key in keys.into_iter() {
        if let YamlValue::Mapping(ref map) = current_node {
            current_node = map
                .get(&YamlValue::String(key.to_string()))
                .ok_or_else(|| {
                    format!(
                        "Key '{}' not found in YAML key path: {}",
                        key, yaml_key_path
                    )
                })?;
        } else {
            return Err(format!("Invalid YAML key path: {}", yaml_key_path).into());
        }
    }

    // Convert the retrieved YAML value to a String
    if let YamlValue::String(value) = current_node {
        Ok(value.clone())
    } else {
        Err("The retrieved YAML value is not a string.".into())
    }
}
