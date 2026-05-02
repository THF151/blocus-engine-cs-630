#![allow(clippy::needless_pass_by_value)]

use pyo3::prelude::*;
use uuid::Uuid;

fn input_config_error() -> PyErr {
    Python::try_attach(|py| {
        crate::errors::input_error(py, blocus_core::InputError::InvalidGameConfig)
    })
    .unwrap_or_else(|| {
        PyErr::new::<pyo3::exceptions::PyRuntimeError, _>("Python interpreter is not attached")
    })
}

pub fn parse_uuid(value: &str, _field_name: &str) -> PyResult<Uuid> {
    Uuid::parse_str(value).map_err(|_error| input_config_error())
}

pub fn parse_three_players(players: Vec<String>) -> PyResult<[blocus_core::PlayerId; 3]> {
    let [first, second, third]: [String; 3] = players
        .try_into()
        .map_err(|_values: Vec<String>| input_config_error())?;

    Ok([
        blocus_core::PlayerId::from_uuid(parse_uuid(&first, "shared_players[0]")?),
        blocus_core::PlayerId::from_uuid(parse_uuid(&second, "shared_players[1]")?),
        blocus_core::PlayerId::from_uuid(parse_uuid(&third, "shared_players[2]")?),
    ])
}

pub fn parse_player_id(value: &str, field_name: &str) -> PyResult<blocus_core::PlayerId> {
    Ok(blocus_core::PlayerId::from_uuid(parse_uuid(
        value, field_name,
    )?))
}

pub fn map_domain_error(error: blocus_core::DomainError) -> PyErr {
    Python::try_attach(|py| match error {
        blocus_core::DomainError::RuleViolation(error) => {
            crate::errors::rule_violation_error(py, error)
        }
        blocus_core::DomainError::InputError(error) => crate::errors::input_error(py, error),
        blocus_core::DomainError::EngineError(error) => crate::errors::engine_error(py, error),
        _ => crate::errors::engine_error(py, blocus_core::EngineError::InvariantViolation),
    })
    .unwrap_or_else(|| {
        PyErr::new::<pyo3::exceptions::PyRuntimeError, _>("Python interpreter is not attached")
    })
}

pub fn map_input_error(error: blocus_core::InputError) -> PyErr {
    Python::try_attach(|py| crate::errors::input_error(py, error)).unwrap_or_else(|| {
        PyErr::new::<pyo3::exceptions::PyRuntimeError, _>("Python interpreter is not attached")
    })
}

pub fn invalid_game_config_error() -> PyErr {
    input_config_error()
}
