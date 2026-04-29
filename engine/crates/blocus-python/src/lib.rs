use pyo3::prelude::*;

#[pyfunction]
fn engine_health() -> bool {
    blocus_core::engine_health()
}

#[pymodule]
fn blocus_engine(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_function(wrap_pyfunction!(engine_health, module)?)?;
    Ok(())
}
