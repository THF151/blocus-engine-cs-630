//! Legal move generation.
//!
//! The iterator evaluates one `(piece, orientation)` at a time and computes all
//! legal anchors for that shape with board-mask algebra. Non-opening moves are
//! constrained to the same-color diagonal frontier up front, avoiding the
//! previous full-board row/column sweep. Iteration order remains stable:
//! `(piece_id, orientation_id, anchor bit index)`.

use crate::pieces::{PieceRepository, ShapeBitmap};
use crate::rules::placement::starting_corner_for;
use crate::{
    BOARD_SIZE, BoardIndex, BoardMask, DomainError, GameState, GameStatus, LegalMove,
    MAX_SHAPE_EXTENT, OrientationId, PIECE_COUNT, PieceId, PlayerColor, PlayerId,
};

/// Lazy legal move iterator.
///
/// The iterator is a snapshot: it copies the state-derived masks and inventory
/// bitmap at construction time, so it does not borrow the source [`GameState`].
#[derive(Clone, Debug)]
pub struct LegalMoveIter {
    repository: &'static PieceRepository,

    target_mask: BoardMask,
    forbidden_cells: BoardMask,
    available_pieces: u32,

    next_piece_id: u8,
    next_orientation_id: u8,
    current_anchor_mask: BoardMask,
    current_piece_id: u8,
    current_orientation_id: u8,
    current_score_delta: u8,
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
        let target_mask = if own_mask.is_empty() {
            BoardMask::from_index(starting_corner_for(color))
        } else {
            own_mask.diagonal_frontier()
        };
        let forbidden_cells = if own_mask.is_empty() {
            occupied_mask
        } else {
            occupied_mask.union(own_mask.edge_neighbors())
        };
        let available_pieces = if context_is_valid {
            state.inventories[color.index()].available_mask()
        } else {
            0
        };

        Self {
            repository,
            target_mask,
            forbidden_cells,
            available_pieces,
            next_piece_id: 0,
            next_orientation_id: 0,
            current_anchor_mask: BoardMask::EMPTY,
            current_piece_id: 0,
            current_orientation_id: 0,
            current_score_delta: 0,
            exhausted: !context_is_valid,
        }
    }

    #[inline]
    fn advance_piece(&mut self) {
        self.next_piece_id = self.next_piece_id.saturating_add(1);
        self.next_orientation_id = 0;
    }

    #[inline]
    fn advance_orientation_cursor(&mut self) {
        self.next_orientation_id = self.next_orientation_id.saturating_add(1);
    }

    fn load_next_anchor_mask(&mut self) -> bool {
        while !self.exhausted {
            if self.next_piece_id >= PIECE_COUNT {
                self.exhausted = true;
                return false;
            }

            if self.available_pieces & (1u32 << self.next_piece_id) == 0 {
                self.advance_piece();
                continue;
            }

            let Ok(piece_id) = PieceId::try_new(self.next_piece_id) else {
                self.exhausted = true;
                return false;
            };

            let piece = self.repository.piece(piece_id);

            if self.next_orientation_id >= piece.orientation_count() {
                self.advance_piece();
                continue;
            }

            let Ok(orientation_id) = OrientationId::try_new(self.next_orientation_id) else {
                self.exhausted = true;
                return false;
            };

            self.advance_orientation_cursor();

            let Some(orientation) = piece.orientation(orientation_id) else {
                continue;
            };

            let shape = orientation.shape();
            let anchor_mask = legal_anchor_mask(shape, self.target_mask, self.forbidden_cells);

            if anchor_mask.is_empty() {
                continue;
            }

            self.current_piece_id = piece_id.as_u8();
            self.current_orientation_id = orientation_id.as_u8();
            self.current_score_delta = shape.square_count();
            self.current_anchor_mask = anchor_mask;
            return true;
        }

        false
    }
}

impl Iterator for LegalMoveIter {
    type Item = LegalMove;

    fn next(&mut self) -> Option<Self::Item> {
        if self.exhausted {
            return None;
        }

        loop {
            if let Some(anchor) = self.current_anchor_mask.pop_lowest_index() {
                let piece_id = PieceId::try_new(self.current_piece_id)
                    .unwrap_or_else(|_| unreachable!("current piece id is valid"));
                let orientation_id = OrientationId::try_new(self.current_orientation_id)
                    .unwrap_or_else(|_| unreachable!("current orientation id is valid"));

                return Some(LegalMove {
                    piece_id,
                    orientation_id,
                    anchor,
                    score_delta: self.current_score_delta,
                });
            }

            if !self.load_next_anchor_mask() {
                return None;
            }
        }
    }
}

fn legal_anchor_mask(
    shape: ShapeBitmap,
    required_cells: BoardMask,
    forbidden_cells: BoardMask,
) -> BoardMask {
    let mut required_anchors = BoardMask::EMPTY;
    let mut forbidden_anchors = BoardMask::EMPTY;
    let mut remaining = shape.cell_mask();

    while remaining != 0 {
        let bit = u8::try_from(remaining.trailing_zeros())
            .unwrap_or_else(|_| unreachable!("shape cell mask uses only 25 bits"));
        let local_row = bit / MAX_SHAPE_EXTENT;
        let local_col = bit % MAX_SHAPE_EXTENT;
        let row_delta = -i8::try_from(local_row)
            .unwrap_or_else(|_| unreachable!("shape row offset fits in i8"));
        let col_delta = -i8::try_from(local_col)
            .unwrap_or_else(|_| unreachable!("shape column offset fits in i8"));

        required_anchors = required_anchors.union(required_cells.shift_by(row_delta, col_delta));
        forbidden_anchors = forbidden_anchors.union(forbidden_cells.shift_by(row_delta, col_delta));
        remaining &= remaining - 1;
    }

    valid_anchor_mask(shape)
        .intersection(required_anchors)
        .difference(forbidden_anchors)
}

fn valid_anchor_mask(shape: ShapeBitmap) -> BoardMask {
    let row_limit = BOARD_SIZE + 1 - shape.height();
    let col_limit = BOARD_SIZE + 1 - shape.width();
    let mut mask = BoardMask::EMPTY;
    let mut row = 0u8;

    while row < row_limit {
        let mut col = 0u8;

        while col < col_limit {
            let index = BoardIndex::from_row_col(row, col)
                .unwrap_or_else(|_| unreachable!("anchor bounds are playable"));
            mask.insert(index);
            col += 1;
        }

        row += 1;
    }

    mask
}

/// Creates a legal move iterator.
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
