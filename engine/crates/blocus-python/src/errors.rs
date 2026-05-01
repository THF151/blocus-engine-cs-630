//! Python exception classes and error mapping helpers.

use blocus_core::InputError as CoreInputError;
use pyo3::create_exception;
use pyo3::exceptions::PyException;
use pyo3::prelude::*;
use pyo3::types::{PyModule, PyType};

create_exception!(blocus_engine, BlocusError, PyException);
create_exception!(blocus_engine, RuleViolationError, BlocusError);
create_exception!(blocus_engine, InputError, BlocusError);
create_exception!(blocus_engine, EngineError, BlocusError);

pub fn register(module: &Bound<'_, PyModule>) -> PyResult<()> {
    let py = module.py();

    module.add("BlocusError", py.get_type::<BlocusError>())?;
    module.add("RuleViolationError", py.get_type::<RuleViolationError>())?;
    module.add("InputError", py.get_type::<InputError>())?;
    module.add("EngineError", py.get_type::<EngineError>())?;

    Ok(())
}

pub fn input_error(py: Python<'_>, error: CoreInputError) -> PyErr {
    make_error(
        &py.get_type::<InputError>(),
        error.code(),
        error.category(),
        error.message(),
    )
}

fn make_error(
    exception_type: &Bound<'_, PyType>,
    code: &'static str,
    category: &'static str,
    message: &'static str,
) -> PyErr {
    let Ok(instance) = exception_type.call1((message,)) else {
        return PyException::new_err(format!("{code}: {message}"));
    };

    if instance.setattr("code", code).is_err()
        || instance.setattr("category", category).is_err()
        || instance.setattr("message", message).is_err()
    {
        return PyException::new_err(format!("{code}: {message}"));
    }

    PyErr::from_value(instance)
}
