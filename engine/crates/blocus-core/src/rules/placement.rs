//! Placement mask construction and placement validation scaffold.

use crate::pieces::{PieceOrientation, PieceRepository};
use crate::{
    BoardIndex, BoardMask, DomainError, GameState, GameStatus, InputError, OrientationId, PieceId,
    PlaceCommand, RuleViolation,
};

/// A concrete board placement of one precomputed piece orientation.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct Placement {
    piece_id: PieceId,
    orientation_id: OrientationId,
    anchor: BoardIndex,
    mask: BoardMask,
    square_count: u8,
}

impl Placement {
    /// Returns the placed piece id.
    #[must_use]
    pub const fn piece_id(self) -> PieceId {
        self.piece_id
    }

    /// Returns the placed orientation id.
    #[must_use]
    pub const fn orientation_id(self) -> OrientationId {
        self.orientation_id
    }

    /// Returns the anchor cell.
    #[must_use]
    pub const fn anchor(self) -> BoardIndex {
        self.anchor
    }

    /// Returns the board mask occupied by this placement.
    #[must_use]
    pub const fn mask(self) -> BoardMask {
        self.mask
    }

    /// Returns the number of occupied squares in this placement.
    #[must_use]
    pub const fn square_count(self) -> u8 {
        self.square_count
    }
}

/// Builds a board placement mask from a precomputed orientation and anchor.
///
/// # Errors
///
/// Returns [`RuleViolation::OutOfBounds`] when any occupied orientation cell
/// would land outside the playable board.
pub fn build_placement(
    piece_id: PieceId,
    orientation: PieceOrientation,
    anchor: BoardIndex,
) -> Result<Placement, DomainError> {
    let shape = orientation.shape();
    let mut mask = BoardMask::EMPTY;

    for (local_row, local_col) in shape.cells() {
        let Some(row) = anchor.row().checked_add(local_row) else {
            return Err(RuleViolation::OutOfBounds.into());
        };
        let Some(col) = anchor.col().checked_add(local_col) else {
            return Err(RuleViolation::OutOfBounds.into());
        };

        let index = BoardIndex::from_row_col(row, col)
            .map_err(|_| DomainError::from(RuleViolation::OutOfBounds))?;

        mask.insert(index);
    }

    Ok(Placement {
        piece_id,
        orientation_id: orientation.id(),
        anchor,
        mask,
        square_count: shape.square_count(),
    })
}

/// Validates the currently implemented placement preconditions.
///
/// This scaffold intentionally stops before corner/edge-contact rules. It
/// validates state/input consistency, ownership, turn, inventory availability,
/// orientation existence, board bounds, and overlap.
///
/// # Errors
///
/// Returns exact domain error variants for the first failed precondition.
pub fn validate_place_command(
    state: &GameState,
    command: PlaceCommand,
    repository: &PieceRepository,
) -> Result<Placement, DomainError> {
    if command.game_id != state.game_id {
        return Err(InputError::GameIdMismatch.into());
    }

    if state.status == GameStatus::Finished {
        return Err(RuleViolation::GameAlreadyFinished.into());
    }

    if !state
        .player_slots
        .can_control_color(command.player_id, command.color)
    {
        return Err(RuleViolation::PlayerDoesNotControlColor.into());
    }

    if state.turn.current_color() != command.color {
        return Err(RuleViolation::WrongPlayerTurn.into());
    }

    if state.inventories[command.color.index()].is_used(command.piece_id) {
        return Err(RuleViolation::PieceAlreadyUsed.into());
    }

    let Some(orientation) = repository
        .piece(command.piece_id)
        .orientation(command.orientation_id)
    else {
        return Err(InputError::UnknownOrientation.into());
    };

    let placement = build_placement(command.piece_id, orientation, command.anchor)?;

    if placement.mask().intersects(state.board.occupied_all()) {
        return Err(RuleViolation::Overlap.into());
    }

    Ok(placement)
}
