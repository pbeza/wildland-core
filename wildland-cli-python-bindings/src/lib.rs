use pyo3::prelude::*;

#[pyfunction]
pub fn get_python_cli_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

#[pyfunction]
pub fn get_admin_manager_version() -> &'static str {
    wildland_admin_manager::get_version()
}

#[pyfunction]
pub fn get_corex_version_verbose() -> String {
    let mut versions = String::new();
    for (name, ver) in wildland_corex::get_version_verbose().iter() {
        versions += &format!("[+] * {name} version {ver}\n");
    }
    versions
}

/// A Python module implemented in Rust.
#[pymodule]
fn wildland_cli_python_bindings(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(get_python_cli_version, m)?)?;
    m.add_function(wrap_pyfunction!(get_admin_manager_version, m)?)?;
    m.add_function(wrap_pyfunction!(get_corex_version_verbose, m)?)?;
    Ok(())
}
