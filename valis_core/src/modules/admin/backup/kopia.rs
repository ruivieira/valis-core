use std::path::PathBuf;

use crate::modules::core::run;
use crate::modules::log::ack;

#[derive(Debug)]
pub struct S3Endpoint {
    pub bucket: String,
    pub access_key: String,
    pub secret_key: String,
    pub endpoint: String,
    pub password: String,
}

/// Connect to a kopia repository on S3.
///
/// # Arguments
///
/// * `s3` - A struct containing the S3 endpoint information.
pub fn kopia_connect_s3(s3: &S3Endpoint) {
    let bucket = s3.bucket.to_owned();
    let access_key = s3.access_key.to_owned();
    let secret_key = s3.secret_key.to_owned();
    let endpoint = s3.endpoint.to_owned();
    let password = s3.password.to_owned();

    run(
        &format!("kopia repository connect s3 {} {} {} {} {}",
                 &format!("--bucket={bucket}"),
                 &format!("--access-key={access_key}"),
                 &format!("--secret-access-key={secret_key}"),
                 &format!("--endpoint={endpoint}"),
                 &format!("--password={password}"))
    )
}

/// Backup a list of locations to a kopia repository on S3.
///
/// # Arguments
///
/// * `s3` - A struct containing the S3 endpoint information.
/// * `locations` - A vector of locations to backup.
pub fn backup(s3: &S3Endpoint, locations: &Vec<PathBuf>) {
    ack(&format!(
        "Backing up to {}@{}",
        s3.bucket, s3.endpoint
    ));
    kopia_connect_s3(s3);
    locations.into_iter().for_each(|location| {
        run(
            &format!("kopia snapshot create {} ", location.to_str().unwrap()),
        )
    })
}