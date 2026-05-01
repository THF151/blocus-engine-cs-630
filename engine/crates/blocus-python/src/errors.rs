use pyo3::create_exception;
use pyo3::prelude::*;
use pyo3::types::PyModule;

use blocus_core::{
    EngineError as CoreEngineError, InputError as CoreInputError,
    RuleViolation as CoreRuleViolation,
};

create_exception!(blocus_engine, BlocusError, pyo3::exceptions::PyException);
create_exception!(blocus_engine, InputError, BlocusError);
create_exception!(blocus_engine, RuleViolationError, BlocusError);
create_exception!(blocus_engine, EngineError, BlocusError);

pub fn register(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add("BlocusError", module.py().get_type::<BlocusError>())?;
    module.add("InputError", module.py().get_type::<InputError>())?;
    module.add(
        "RuleViolationError",
        module.py().get_type::<RuleViolationError>(),
    )?;
    module.add("EngineError", module.py().get_type::<EngineError>())?;
    Ok(())
}

fn structured_message(category: &str, code: &str, message: &str) -> String {
    format!("{category}:{code}:{message}")
}

pub fn input_error(py: Python<'_>, error: CoreInputError) -> PyErr {
    let _ = py;
    PyErr::new::<InputError, _>(structured_message(
        error.category(),
        error.code(),
        error.message(),
    ))
}

pub fn rule_violation_error(py: Python<'_>, error: CoreRuleViolation) -> PyErr {
    let _ = py;
    PyErr::new::<RuleViolationError, _>(structured_message(
        error.category(),
        error.code(),
        error.message(),
    ))
}

pub fn engine_error(py: Python<'_>, error: CoreEngineError) -> PyErr {
    let _ = py;
    PyErr::new::<EngineError, _>(structured_message(
        error.category(),
        error.code(),
        error.message(),
    ))
}
