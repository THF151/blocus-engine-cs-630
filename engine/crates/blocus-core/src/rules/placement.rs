//! Placement construction and rule validation.

use crate::pieces::{PieceOrientation, PieceRepository};
use crate::{
    BOARD_SIZE, BoardIndex, BoardMask, DomainError, GameState, GameStatus, InputError,
    OrientationId, PIECE_COUNT, PieceId, PlaceCommand, PlayerColor, RuleViolation,
};

/// A concrete piece placement on the board.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct Placement {
    piece_id: PieceId,
    orientation_id: OrientationId,
    anchor: BoardIndex,
    mask: BoardMask,
    square_count: u8,
}

impl Placement {
    #[must_use]
    pub const fn new(
        piece_id: PieceId,
        orientation_id: OrientationId,
        anchor: BoardIndex,
        mask: BoardMask,
        square_count: u8,
    ) -> Self {
        Self {
            piece_id,
            orientation_id,
            anchor,
            mask,
            square_count,
        }
    }

    #[must_use]
    pub const fn piece_id(self) -> PieceId {
        self.piece_id
    }

    #[must_use]
    pub const fn orientation_id(self) -> OrientationId {
        self.orientation_id
    }

    #[must_use]
    pub const fn anchor(self) -> BoardIndex {
        self.anchor
    }

    #[must_use]
    pub const fn mask(self) -> BoardMask {
        self.mask
    }

    #[must_use]
    pub const fn square_count(self) -> u8 {
        self.square_count
    }
}

/// Builds a board mask for an already-resolved piece orientation.
///
/// # Errors
///
/// Returns [`RuleViolation::OutOfBounds`] if any occupied orientation cell would
/// land outside the playable board.
pub fn build_placement(
    piece_id: PieceId,
    orientation: PieceOrientation,
    anchor: BoardIndex,
) -> Result<Placement, DomainError> {
    let shape = orientation.shape();
    let anchor_row = anchor.row();
    let anchor_col = anchor.col();

    let mut mask = BoardMask::EMPTY;

    for (local_row, local_col) in shape.cells() {
        let Some(row) = anchor_row.checked_add(local_row) else {
            return Err(RuleViolation::OutOfBounds.into());
        };

        let Some(col) = anchor_col.checked_add(local_col) else {
            return Err(RuleViolation::OutOfBounds.into());
        };

        if row >= BOARD_SIZE || col >= BOARD_SIZE {
            return Err(RuleViolation::OutOfBounds.into());
        }

        let Ok(index) = BoardIndex::from_row_col(row, col) else {
            return Err(RuleViolation::OutOfBounds.into());
        };

        mask.insert(index);
    }

    Ok(Placement::new(
        piece_id,
        orientation.id(),
        anchor,
        mask,
        shape.square_count(),
    ))
}

/// Validates a place command against state and a piece repository.
///
/// This validates command/state identity, player ownership, turn, piece
/// availability, orientation existence, placement bounds, overlap, first-move
/// starting-corner coverage, same-color edge rejection, and same-color diagonal
/// contact for non-first moves.
///
/// # Errors
///
/// Returns typed domain errors for malformed or illegal placements.
pub fn validate_place_command(
    state: &GameState,
    command: PlaceCommand,
    repository: &PieceRepository,
) -> Result<Placement, DomainError> {
    if state.status != GameStatus::InProgress {
        return Err(RuleViolation::GameAlreadyFinished.into());
    }

    if command.game_id != state.game_id {
        return Err(InputError::GameIdMismatch.into());
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

    if usize::from(command.piece_id.as_u8()) >= usize::from(PIECE_COUNT) {
        return Err(InputError::UnknownPiece.into());
    }

    if state.inventories[command.color.index()].is_used(command.piece_id) {
        return Err(RuleViolation::PieceAlreadyUsed.into());
    }

    let piece = repository.piece(command.piece_id);

    let Some(orientation) = piece.orientation(command.orientation_id) else {
        return Err(InputError::UnknownOrientation.into());
    };

    let placement = build_placement(command.piece_id, orientation, command.anchor)?;

    if placement.mask().intersects(state.board.occupied_all()) {
        return Err(RuleViolation::Overlap.into());
    }

    validate_contact_rules(state, command.color, placement)?;

    Ok(placement)
}

fn validate_contact_rules(
    state: &GameState,
    color: PlayerColor,
    placement: Placement,
) -> Result<(), DomainError> {
    let own_mask = state.board.occupied(color);

    if own_mask.is_empty() {
        return validate_first_move_covers_starting_corner(color, placement);
    }

    if has_same_color_edge_contact(placement.mask(), own_mask) {
        return Err(RuleViolation::IllegalEdgeContact.into());
    }

    if !has_same_color_corner_contact(placement.mask(), own_mask) {
        return Err(RuleViolation::MissingCornerContact.into());
    }

    Ok(())
}

fn validate_first_move_covers_starting_corner(
    color: PlayerColor,
    placement: Placement,
) -> Result<(), DomainError> {
    let corner = starting_corner_for(color);

    if placement.mask().contains(corner) {
        Ok(())
    } else {
        Err(RuleViolation::MissingCornerContact.into())
    }
}

fn has_same_color_edge_contact(placement_mask: BoardMask, own_mask: BoardMask) -> bool {
    has_neighbor_contact(
        placement_mask,
        own_mask,
        &[(0, -1), (-1, 0), (0, 1), (1, 0)],
    )
}

fn has_same_color_corner_contact(placement_mask: BoardMask, own_mask: BoardMask) -> bool {
    has_neighbor_contact(
        placement_mask,
        own_mask,
        &[(-1, -1), (-1, 1), (1, -1), (1, 1)],
    )
}

fn has_neighbor_contact(
    placement_mask: BoardMask,
    own_mask: BoardMask,
    offsets: &[(i8, i8)],
) -> bool {
    for row in 0..BOARD_SIZE {
        for col in 0..BOARD_SIZE {
            let index = playable_index(row, col);

            if !placement_mask.contains(index) {
                continue;
            }

            for &(row_delta, col_delta) in offsets {
                let Some(neighbor) = offset_index(row, col, row_delta, col_delta) else {
                    continue;
                };

                if own_mask.contains(neighbor) {
                    return true;
                }
            }
        }
    }

    false
}

fn offset_index(row: u8, col: u8, row_delta: i8, col_delta: i8) -> Option<BoardIndex> {
    let next_row = i16::from(row) + i16::from(row_delta);
    let next_col = i16::from(col) + i16::from(col_delta);

    if !(0..i16::from(BOARD_SIZE)).contains(&next_row)
        || !(0..i16::from(BOARD_SIZE)).contains(&next_col)
    {
        return None;
    }

    let Ok(next_row) = u8::try_from(next_row) else {
        return None;
    };

    let Ok(next_col) = u8::try_from(next_col) else {
        return None;
    };

    Some(playable_index(next_row, next_col))
}

fn starting_corner_for(color: PlayerColor) -> BoardIndex {
    let (row, col) = match color {
        PlayerColor::Blue => (0, 0),
        PlayerColor::Yellow => (0, BOARD_SIZE - 1),
        PlayerColor::Red => (BOARD_SIZE - 1, BOARD_SIZE - 1),
        PlayerColor::Green => (BOARD_SIZE - 1, 0),
    };

    playable_index(row, col)
}

fn playable_index(row: u8, col: u8) -> BoardIndex {
    BoardIndex::from_row_col(row, col)
        .unwrap_or_else(|_| unreachable!("validated playable row and column are always valid"))
}
