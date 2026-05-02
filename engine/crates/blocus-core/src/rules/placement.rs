//! Placement construction and rule validation.

use crate::pieces::{PieceOrientation, PieceRepository, ShapeBitmap};
use crate::{
    BOARD_LANES, BOARD_SIZE, BoardIndex, BoardMask, DomainError, GameState, GameStatus, InputError,
    MAX_SHAPE_EXTENT, OrientationId, PIECE_COUNT, PieceId, PlaceCommand, PlayerColor, ROW_STRIDE,
    RuleViolation,
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

/// Builds a board mask from a normalized shape and an anchor without
/// allocating.
///
/// Returns `None` if any cell of the placement would land outside the playable
/// board. Iterates only the set bits of the shape's compact `5 × 5` cell mask,
/// so it does at most five cell insertions per call.
#[must_use]
#[inline]
pub(crate) fn build_placement_mask(shape: ShapeBitmap, anchor: BoardIndex) -> Option<BoardMask> {
    let anchor_row = anchor.row();
    let anchor_col = anchor.col();

    if anchor_row.checked_add(shape.height())? > BOARD_SIZE {
        return None;
    }

    if anchor_col.checked_add(shape.width())? > BOARD_SIZE {
        return None;
    }

    let mut lanes = [0u128; BOARD_LANES];
    let mut remaining = shape.cell_mask();

    while remaining != 0 {
        let bit = u8::try_from(remaining.trailing_zeros())
            .unwrap_or_else(|_| unreachable!("shape cell mask uses only 25 bits"));

        let local_row = bit / MAX_SHAPE_EXTENT;
        let local_col = bit % MAX_SHAPE_EXTENT;
        let global_row = anchor_row + local_row;
        let global_col = anchor_col + local_col;

        let bit_index = u32::from(global_row) * u32::from(ROW_STRIDE) + u32::from(global_col);
        let lane_idx = usize::try_from(bit_index / u128::BITS)
            .unwrap_or_else(|_| unreachable!("board lane index always fits in usize"));
        let lane_offset = bit_index % u128::BITS;

        lanes[lane_idx] |= 1u128 << lane_offset;
        remaining &= remaining - 1;
    }

    Some(BoardMask::from_lanes(lanes))
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

    let Some(mask) = build_placement_mask(shape, anchor) else {
        return Err(RuleViolation::OutOfBounds.into());
    };

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

    if state.turn.current_color() != command.color {
        return Err(RuleViolation::WrongPlayerTurn.into());
    }

    if !state
        .turn
        .is_active_controller(state.player_slots, command.player_id)
    {
        return Err(RuleViolation::PlayerDoesNotControlColor.into());
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
    let occupied_all = state.board.occupied_all();

    if placement.mask().intersects(occupied_all) {
        return Err(RuleViolation::Overlap.into());
    }

    let own_mask = state.board.occupied(command.color);
    validate_contact_rules(command.color, placement.mask(), own_mask)?;

    Ok(placement)
}

#[inline]
fn validate_contact_rules(
    color: PlayerColor,
    placement_mask: BoardMask,
    own_mask: BoardMask,
) -> Result<(), DomainError> {
    if own_mask.is_empty() {
        return validate_first_move_covers_starting_corner(color, placement_mask);
    }

    if has_same_color_edge_contact(placement_mask, own_mask) {
        return Err(RuleViolation::IllegalEdgeContact.into());
    }

    if !has_same_color_corner_contact(placement_mask, own_mask) {
        return Err(RuleViolation::MissingCornerContact.into());
    }

    Ok(())
}

#[inline]
fn validate_first_move_covers_starting_corner(
    color: PlayerColor,
    placement_mask: BoardMask,
) -> Result<(), DomainError> {
    if placement_mask.contains(starting_corner_for(color)) {
        Ok(())
    } else {
        Err(RuleViolation::MissingCornerContact.into())
    }
}

#[inline]
fn has_same_color_edge_contact(placement: BoardMask, own: BoardMask) -> bool {
    placement.shift_north().intersects(own)
        || placement.shift_south().intersects(own)
        || placement.shift_east().intersects(own)
        || placement.shift_west().intersects(own)
}

#[inline]
fn has_same_color_corner_contact(placement: BoardMask, own: BoardMask) -> bool {
    let north = placement.shift_north();
    let south = placement.shift_south();

    north.shift_east().intersects(own)
        || north.shift_west().intersects(own)
        || south.shift_east().intersects(own)
        || south.shift_west().intersects(own)
}

#[inline]
pub(crate) fn starting_corner_for(color: PlayerColor) -> BoardIndex {
    let (row, col) = match color {
        PlayerColor::Blue => (0, 0),
        PlayerColor::Yellow => (0, BOARD_SIZE - 1),
        PlayerColor::Red => (BOARD_SIZE - 1, BOARD_SIZE - 1),
        PlayerColor::Green => (BOARD_SIZE - 1, 0),
    };

    BoardIndex::from_row_col(row, col)
        .unwrap_or_else(|_| unreachable!("configured starting corners are always playable"))
}
