mod command;
mod errors;
mod types;

use pyo3::prelude::*;

#[pyfunction]
fn engine_health() -> bool {
    blocus_core::engine_health()
}

#[pymodule]
fn blocus_engine(module: &Bound<'_, PyModule>) -> PyResult<()> {
    errors::register(module)?;
    types::register(module)?;
    command::register(module)?;

    module.add_function(wrap_pyfunction!(engine_health, module)?)?;

    Ok(())
}
