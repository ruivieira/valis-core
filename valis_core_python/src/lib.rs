use std::env;

use pyo3::{wrap_pyfunction, wrap_pymodule};
use pyo3::prelude::*;

use valis_core::modules::admin::backup::kopia as kopia;
use valis_core::modules::core as core;
use valis_core::modules::log as logrs;

#[pyfunction]
fn ack(message: &str) {
    logrs::ack(message);
}

#[pyfunction]
fn kopia_connect_s3_from_env() {
    let bucket = env::var("WASABI_KOPIA_BUCKET").unwrap();
    let access_key = env::var("WASABI_KOPIA_ACCESS_KEY").unwrap();
    let secret_key = env::var("WASABI_KOPIA_SECRET_KEY").unwrap();
    let endpoint = env::var("WASABI_KOPIA_ENDPOINT").unwrap();
    let password = env::var("KOPIA_PASSWORD").unwrap();
    kopia_connect_s3(&bucket, &access_key, &secret_key, &endpoint, &password)
}


#[pyfunction]
fn kopia_connect_s3(bucket: &str, access_key: &str, secret_key: &str, endpoint: &str, password: &str) {
    let s3_endpoint = kopia::S3Endpoint {
        bucket: bucket.to_string(),
        access_key: access_key.to_string(),
        secret_key: secret_key.to_string(),
        endpoint: endpoint.to_string(),
        password: password.to_string(),
    };
    kopia::kopia_connect_s3(&s3_endpoint);
}

#[pyfunction]
fn create_snapshot(path: &str) {
    kopia::create_snapshot(&core::to_path_buf(path).unwrap());
}


#[pymodule]
fn log(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_wrapped(wrap_pyfunction!(ack))?;
    Ok(())
}

#[pymodule]
fn backup(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_wrapped(wrap_pyfunction!(kopia_connect_s3))?;
    m.add_wrapped(wrap_pyfunction!(kopia_connect_s3_from_env))?;
    m.add_wrapped(wrap_pyfunction!(create_snapshot))?;
    Ok(())
}


#[pymodule]
fn admin(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_wrapped(wrap_pymodule!(backup))?;
    Ok(())
}


#[pymodule]
fn valis_core_python(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_wrapped(wrap_pymodule!(log))?;
    m.add_wrapped(wrap_pymodule!(admin))?;
    Ok(())
}