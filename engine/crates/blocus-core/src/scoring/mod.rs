//! Final scoring.
//!
//! Basic scoring returns the number of remaining squares; lower is better.
//! Advanced scoring returns rulebook points; higher is better.

use crate::pieces::{PieceInventory, PieceRepository};
use crate::{
    DomainError, GameMode, GameState, GameStatus, InputError, PIECE_COUNT, PieceId, PlayerColor,
    PlayerId, RuleViolation, ScoreBoard, ScoreEntry, ScoringMode,
};

/// Scores a finished game.
///
/// # Errors
///
/// Returns [`RuleViolation::GameNotFinished`] if scoring is requested before
/// the game has finished.
pub fn score_game(
    state: &GameState,
    repository: &PieceRepository,
    scoring: ScoringMode,
) -> Result<ScoreBoard, DomainError> {
    if state.mode == GameMode::Duo
        && (state.scoring != ScoringMode::Advanced
            || scoring != ScoringMode::Advanced
            || scoring != state.scoring)
    {
        return Err(InputError::InvalidGameConfig.into());
    }

    if state.status != GameStatus::Finished {
        return Err(RuleViolation::GameNotFinished.into());
    }

    let mut entries: Vec<ScoreEntry> = Vec::with_capacity(state.mode.player_count());

    match state.mode {
        GameMode::TwoPlayer => {
            for color in state.mode.active_colors().iter().copied() {
                let Some(player_id) = state.player_slots.controller_of(color) else {
                    continue;
                };

                let score = score_color(state, repository, color, scoring);
                add_or_update_entry(&mut entries, player_id, score);
            }
        }
        GameMode::ThreePlayer => {
            let shared_color = state.player_slots.shared_color();

            for color in state.mode.active_colors().iter().copied() {
                if Some(color) == shared_color {
                    continue;
                }

                let Some(player_id) = state.player_slots.controller_of(color) else {
                    continue;
                };

                let score = score_color(state, repository, color, scoring);
                entries.push(ScoreEntry { player_id, score });
            }
        }
        GameMode::FourPlayer | GameMode::Duo => {
            for color in state.mode.active_colors().iter().copied() {
                let Some(player_id) = state.player_slots.controller_of(color) else {
                    continue;
                };

                let score = score_color(state, repository, color, scoring);
                entries.push(ScoreEntry { player_id, score });
            }
        }
    }

    Ok(ScoreBoard { scoring, entries })
}

fn add_or_update_entry(entries: &mut Vec<ScoreEntry>, player_id: PlayerId, score_delta: i16) {
    if let Some(entry) = entries
        .iter_mut()
        .find(|entry| entry.player_id == player_id)
    {
        entry.score += score_delta;
    } else {
        entries.push(ScoreEntry {
            player_id,
            score: score_delta,
        });
    }
}

fn score_color(
    state: &GameState,
    repository: &PieceRepository,
    color: PlayerColor,
    scoring: ScoringMode,
) -> i16 {
    let inventory = state.inventories[color.index()];
    let remaining = remaining_square_count(inventory, repository);

    match scoring {
        ScoringMode::Basic => remaining,
        ScoringMode::Advanced => advanced_score_for_color(state, color, inventory, remaining),
    }
}

fn advanced_score_for_color(
    state: &GameState,
    color: PlayerColor,
    inventory: PieceInventory,
    remaining: i16,
) -> i16 {
    if !inventory.is_complete() {
        return -remaining;
    }

    let monomino =
        PieceId::try_new(0).unwrap_or_else(|_| unreachable!("piece id 0 is always valid"));

    if state.last_piece_by_color.get(color) == Some(monomino) {
        20
    } else {
        15
    }
}

fn remaining_square_count(inventory: PieceInventory, repository: &PieceRepository) -> i16 {
    let mut total = 0i16;

    for raw_piece_id in 0..PIECE_COUNT {
        let piece_id = PieceId::try_new(raw_piece_id)
            .unwrap_or_else(|_| unreachable!("piece id in 0..PIECE_COUNT is valid"));

        if inventory.is_available(piece_id) {
            total += i16::from(repository.piece(piece_id).square_count());
        }
    }

    total
}
