mod command;
mod config;
mod conversion;
mod engine;
mod errors;
mod state;
mod types;

use pyo3::prelude::*;

#[pyfunction]
fn engine_health() -> bool {
    blocus_core::engine_health()
}

#[pymodule]
fn blocus_engine(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_function(wrap_pyfunction!(engine_health, module)?)?;
    errors::register(module)?;

    module.add_class::<types::PlayerColor>()?;
    module.add_class::<types::GameStatus>()?;
    module.add_class::<types::ScoringMode>()?;

    command::register(module)?;

    module.add_class::<config::GameMode>()?;
    module.add_class::<config::SharedColorTurn>()?;
    module.add_class::<config::PlayerSlots>()?;
    module.add_class::<config::GameConfig>()?;

    module.add_class::<state::GameState>()?;
    module.add_class::<engine::BlocusEngine>()?;

    let py = module.py();

    let player_color = module.getattr("PlayerColor")?;
    player_color.setattr(
        "BLUE",
        Py::new(
            py,
            types::PlayerColor::from_core(blocus_core::PlayerColor::Blue),
        )?,
    )?;
    player_color.setattr(
        "YELLOW",
        Py::new(
            py,
            types::PlayerColor::from_core(blocus_core::PlayerColor::Yellow),
        )?,
    )?;
    player_color.setattr(
        "RED",
        Py::new(
            py,
            types::PlayerColor::from_core(blocus_core::PlayerColor::Red),
        )?,
    )?;
    player_color.setattr(
        "GREEN",
        Py::new(
            py,
            types::PlayerColor::from_core(blocus_core::PlayerColor::Green),
        )?,
    )?;

    let game_status = module.getattr("GameStatus")?;
    game_status.setattr(
        "IN_PROGRESS",
        Py::new(
            py,
            types::GameStatus::from_core(blocus_core::GameStatus::InProgress),
        )?,
    )?;
    game_status.setattr(
        "FINISHED",
        Py::new(
            py,
            types::GameStatus::from_core(blocus_core::GameStatus::Finished),
        )?,
    )?;

    let scoring_mode = module.getattr("ScoringMode")?;
    scoring_mode.setattr(
        "BASIC",
        Py::new(
            py,
            types::ScoringMode::from_core(blocus_core::ScoringMode::Basic),
        )?,
    )?;
    scoring_mode.setattr(
        "ADVANCED",
        Py::new(
            py,
            types::ScoringMode::from_core(blocus_core::ScoringMode::Advanced),
        )?,
    )?;

    let game_mode = module.getattr("GameMode")?;
    game_mode.setattr(
        "TWO_PLAYER",
        Py::new(
            py,
            config::GameMode::from_core(blocus_core::GameMode::TwoPlayer),
        )?,
    )?;
    game_mode.setattr(
        "THREE_PLAYER",
        Py::new(
            py,
            config::GameMode::from_core(blocus_core::GameMode::ThreePlayer),
        )?,
    )?;
    game_mode.setattr(
        "FOUR_PLAYER",
        Py::new(
            py,
            config::GameMode::from_core(blocus_core::GameMode::FourPlayer),
        )?,
    )?;

    Ok(())
}
