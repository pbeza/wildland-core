use pyo3::prelude::*;

#[pyfunction]
fn get_version() -> PyResult<String> {
    Ok(wildland_admin_manager::get_version().into())
}

/// A Python module implemented in Rust.
#[pymodule]
fn wildland_admin_manager_python(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(get_version, m)?)?;
    Ok(())
}
