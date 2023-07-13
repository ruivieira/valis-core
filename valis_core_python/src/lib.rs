use pyo3::{wrap_pyfunction, wrap_pymodule};
use pyo3::prelude::*;

use valis_core::modules::log as logrs;

#[pyfunction]
fn ack(message: &str) {
    logrs::ack(message);
}


#[pymodule]
fn log(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_wrapped(wrap_pyfunction!(ack))?;
    Ok(())
}

#[pymodule]
fn valis_core_python(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_wrapped(wrap_pymodule!(log))?;
    Ok(())
}