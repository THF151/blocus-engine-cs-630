//! Legal move generation.
//!
//! The iterator is brute-force in shape but optimized for the per-anchor
//! hot path: state-level invariants are validated once at construction,
//! per-orientation bounding boxes constrain the anchor sweep, and contact
//! rules are evaluated through 640-bit mask shifts rather than per-cell
//! scans. Iteration order is `(piece_id, orientation_id, row, col)` and
//! is stable.

use crate::pieces::PieceRepository;
use crate::rules::placement::{build_placement_mask, starting_corner_for};
use crate::{
    BOARD_SIZE, BoardIndex, BoardMask, DomainError, GameState, GameStatus, LegalMove,
    OrientationId, PIECE_COUNT, PieceId, PlayerColor, PlayerId,
};

/// Lazy brute-force legal move iterator.
///
/// The iterator is a snapshot: it copies the state-derived masks and inventory
/// bitmap at construction time, so it does not borrow the source [`GameState`].
#[derive(Clone, Debug)]
pub struct LegalMoveIter {
    repository: &'static PieceRepository,

    own_mask: BoardMask,
    occupied_mask: BoardMask,
    available_pieces: u32,
    is_first_move: bool,
    starting_corner: BoardIndex,

    next_piece_id: u8,
    next_orientation_id: u8,
    next_row: u8,
    next_col: u8,
    exhausted: bool,
}

impl LegalMoveIter {
    /// Creates a new lazy legal move iterator.
    ///
    /// Constructs an exhausted iterator when the gameplay context is invalid:
    /// finished game, wrong color's turn, or a player who is not scheduled to
    /// act for the requested color.
    #[must_use]
    pub fn new(
        state: &GameState,
        repository: &'static PieceRepository,
        player_id: PlayerId,
        color: PlayerColor,
    ) -> Self {
        let context_is_valid = state.status == GameStatus::InProgress
            && state.turn.current_color() == color
            && state
                .turn
                .is_active_controller(state.player_slots, player_id);

        let own_mask = state.board.occupied(color);
        let occupied_mask = state.board.occupied_all();
        let available_pieces = if context_is_valid {
            state.inventories[color.index()].available_mask()
        } else {
            0
        };

        Self {
            repository,
            own_mask,
            occupied_mask,
            available_pieces,
            is_first_move: own_mask.is_empty(),
            starting_corner: starting_corner_for(color),
            next_piece_id: 0,
            next_orientation_id: 0,
            next_row: 0,
            next_col: 0,
            exhausted: !context_is_valid,
        }
    }

    #[inline]
    fn advance_piece(&mut self) {
        self.next_piece_id = self.next_piece_id.saturating_add(1);
        self.next_orientation_id = 0;
        self.next_row = 0;
        self.next_col = 0;
    }

    #[inline]
    fn advance_orientation(&mut self) {
        self.next_orientation_id = self.next_orientation_id.saturating_add(1);
        self.next_row = 0;
        self.next_col = 0;
    }

    #[inline]
    fn placement_is_legal(&self, placement: BoardMask) -> bool {
        if placement.intersects(self.occupied_mask) {
            return false;
        }

        if self.is_first_move {
            return placement.contains(self.starting_corner);
        }

        if has_edge_contact(placement, self.own_mask) {
            return false;
        }

        has_corner_contact(placement, self.own_mask)
    }
}

impl Iterator for LegalMoveIter {
    type Item = LegalMove;

    fn next(&mut self) -> Option<Self::Item> {
        if self.exhausted {
            return None;
        }

        loop {
            if self.next_piece_id >= PIECE_COUNT {
                self.exhausted = true;
                return None;
            }

            if self.available_pieces & (1u32 << self.next_piece_id) == 0 {
                self.advance_piece();
                continue;
            }

            let Ok(piece_id) = PieceId::try_new(self.next_piece_id) else {
                self.exhausted = true;
                return None;
            };

            let piece = self.repository.piece(piece_id);

            if self.next_orientation_id >= piece.orientation_count() {
                self.advance_piece();
                continue;
            }

            let Ok(orientation_id) = OrientationId::try_new(self.next_orientation_id) else {
                self.exhausted = true;
                return None;
            };

            let Some(orientation) = piece.orientation(orientation_id) else {
                self.advance_orientation();
                continue;
            };

            let shape = orientation.shape();

            let max_anchor_row = (BOARD_SIZE + 1).saturating_sub(shape.height());
            let max_anchor_col = (BOARD_SIZE + 1).saturating_sub(shape.width());

            if self.next_row >= max_anchor_row {
                self.advance_orientation();
                continue;
            }

            if self.next_col >= max_anchor_col {
                self.next_col = 0;
                self.next_row = self.next_row.saturating_add(1);
                continue;
            }

            let row = self.next_row;
            let col = self.next_col;
            self.next_col = self.next_col.saturating_add(1);

            let Ok(anchor) = BoardIndex::from_row_col(row, col) else {
                continue;
            };

            let Some(placement_mask) = build_placement_mask(shape, anchor) else {
                continue;
            };

            if !self.placement_is_legal(placement_mask) {
                continue;
            }

            return Some(LegalMove {
                piece_id,
                orientation_id,
                anchor,
                score_delta: shape.square_count(),
            });
        }
    }
}

#[inline]
fn has_edge_contact(placement: BoardMask, own: BoardMask) -> bool {
    placement.shift_north().intersects(own)
        || placement.shift_south().intersects(own)
        || placement.shift_east().intersects(own)
        || placement.shift_west().intersects(own)
}

#[inline]
fn has_corner_contact(placement: BoardMask, own: BoardMask) -> bool {
    let north = placement.shift_north();
    let south = placement.shift_south();

    north.shift_east().intersects(own)
        || north.shift_west().intersects(own)
        || south.shift_east().intersects(own)
        || south.shift_west().intersects(own)
}

/// Creates a brute-force legal move iterator.
///
/// # Errors
///
/// Returns [`crate::EngineError::CorruptedState`] if the supplied state violates
/// compact board or turn invariants.
pub fn legal_moves_iter(
    state: &GameState,
    repository: &'static PieceRepository,
    player_id: PlayerId,
    color: PlayerColor,
) -> Result<LegalMoveIter, DomainError> {
    crate::validate_game_state(state)?;

    Ok(LegalMoveIter::new(state, repository, player_id, color))
}

/// Collects all legal moves from the lazy iterator.
///
/// # Errors
///
/// Propagates iterator-construction errors.
pub fn get_valid_moves(
    state: &GameState,
    repository: &'static PieceRepository,
    player_id: PlayerId,
    color: PlayerColor,
) -> Result<Vec<LegalMove>, DomainError> {
    Ok(legal_moves_iter(state, repository, player_id, color)?.collect())
}

/// Collects all legal moves for one specific piece.
///
/// # Errors
///
/// Propagates iterator-construction errors.
pub fn get_valid_moves_for_piece(
    state: &GameState,
    repository: &'static PieceRepository,
    player_id: PlayerId,
    color: PlayerColor,
    piece_id: PieceId,
) -> Result<Vec<LegalMove>, DomainError> {
    Ok(legal_moves_iter(state, repository, player_id, color)?
        .filter(|legal_move| legal_move.piece_id == piece_id)
        .collect())
}

/// Returns whether at least one legal move exists.
///
/// # Errors
///
/// Propagates iterator-construction errors.
pub fn has_any_valid_move(
    state: &GameState,
    repository: &'static PieceRepository,
    player_id: PlayerId,
    color: PlayerColor,
) -> Result<bool, DomainError> {
    Ok(legal_moves_iter(state, repository, player_id, color)?
        .next()
        .is_some())
}

/// Returns whether at least one legal move exists for one specific piece.
///
/// # Errors
///
/// Propagates iterator-construction errors.
pub fn has_any_valid_move_for_piece(
    state: &GameState,
    repository: &'static PieceRepository,
    player_id: PlayerId,
    color: PlayerColor,
    piece_id: PieceId,
) -> Result<bool, DomainError> {
    Ok(legal_moves_iter(state, repository, player_id, color)?
        .any(|legal_move| legal_move.piece_id == piece_id))
}
