//! Public state-related API DTOs.

use crate::pieces::PieceInventory;
use crate::{
    BoardState, GameId, GameMode, PIECE_COUNT, PLAYER_COLOR_COUNT, PieceId, PlayerColor,
    PlayerSlots, StateVersion, TurnOrder, TurnState, ZobristHash,
};

/// Current serialized state schema version.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct StateSchemaVersion(u16);

impl StateSchemaVersion {
    /// Current state schema version.
    pub const CURRENT: Self = Self(1);

    /// Creates a schema version from a raw value.
    #[must_use]
    pub const fn new(value: u16) -> Self {
        Self(value)
    }

    /// Returns the raw schema version.
    #[must_use]
    pub const fn as_u16(self) -> u16 {
        self.0
    }
}

impl Default for StateSchemaVersion {
    fn default() -> Self {
        Self::CURRENT
    }
}

impl From<StateSchemaVersion> for u16 {
    fn from(value: StateSchemaVersion) -> Self {
        value.0
    }
}

/// Current game status.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[non_exhaustive]
pub enum GameStatus {
    /// Game is active.
    InProgress,
    /// Game has finished.
    Finished,
}

/// Scoring mode used for final scoring.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[non_exhaustive]
pub enum ScoringMode {
    /// Basic scoring: fewer remaining squares wins.
    Basic,
    /// Advanced scoring: remaining squares are negative, with completion
    /// bonuses.
    Advanced,
}

/// Compact tracking for the last placed piece per color.
///
/// Each color uses one five-bit slot:
///
/// - `0` means no placed piece has been recorded.
/// - `1..=21` stores `piece_id + 1`.
///
/// This keeps [`GameState`] below the compact-state size budget while still
/// supporting the advanced-scoring monomino-last bonus.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Hash)]
pub struct LastPieceByColor {
    packed: u32,
}

impl LastPieceByColor {
    /// Empty last-piece tracking.
    pub const EMPTY: Self = Self { packed: 0 };

    /// Creates the compact tracker from a raw packed value.
    ///
    /// Bits outside the four five-bit slots are ignored.
    #[must_use]
    pub const fn from_packed(packed: u32) -> Self {
        Self {
            packed: packed & LAST_PIECE_PACKED_MASK,
        }
    }

    /// Returns the raw packed representation.
    #[must_use]
    pub const fn packed(self) -> u32 {
        self.packed
    }

    /// Returns the last placed piece for a color, if one has been recorded.
    #[must_use]
    pub fn get(self, color: PlayerColor) -> Option<PieceId> {
        let slot = (self.packed >> last_piece_shift(color)) & LAST_PIECE_SLOT_MASK;

        if slot == 0 {
            None
        } else {
            PieceId::try_new(u8::try_from(slot - 1).unwrap_or_else(|_| {
                unreachable!("last-piece slot stores only values in the range 1..=21")
            }))
            .ok()
        }
    }

    /// Records the last placed piece for a color.
    pub fn set(&mut self, color: PlayerColor, piece_id: PieceId) {
        let shift = last_piece_shift(color);
        let clear_mask = !(LAST_PIECE_SLOT_MASK << shift);
        let encoded = u32::from(piece_id.as_u8()) + 1;

        self.packed = (self.packed & clear_mask) | (encoded << shift);
    }

    /// Returns a copy with the last placed piece recorded for a color.
    #[must_use]
    pub fn with_set(mut self, color: PlayerColor, piece_id: PieceId) -> Self {
        self.set(color, piece_id);
        self
    }
}

/// Public game-state DTO.
///
/// This is a value object. It deliberately stores compact domain primitives
/// instead of Python-facing wrapper objects.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct GameState {
    /// State schema version.
    pub schema_version: StateSchemaVersion,
    /// Game identifier.
    pub game_id: GameId,
    /// Game mode.
    pub mode: GameMode,
    /// Scoring mode.
    pub scoring: ScoringMode,
    /// Game-specific turn order.
    pub turn_order: TurnOrder,
    /// Player/color assignment.
    pub player_slots: PlayerSlots,
    /// Board occupancy state.
    pub board: BoardState,
    /// Per-color inventories.
    pub inventories: [PieceInventory; PLAYER_COLOR_COUNT],
    /// Last placed piece by color, used for advanced scoring bonuses.
    pub last_piece_by_color: LastPieceByColor,
    /// Turn progression state.
    pub turn: TurnState,
    /// Game status.
    pub status: GameStatus,
    /// Monotonic state version.
    pub version: StateVersion,
    /// Semantic state hash placeholder.
    pub hash: ZobristHash,
}

impl GameState {
    /// Returns used piece ids for a color in ascending canonical piece order.
    #[must_use]
    pub fn used_piece_ids(&self, color: PlayerColor) -> Vec<PieceId> {
        piece_ids_matching_inventory(self.inventories[color.index()], true)
    }

    /// Returns available piece ids for a color in ascending canonical piece order.
    #[must_use]
    pub fn available_piece_ids(&self, color: PlayerColor) -> Vec<PieceId> {
        piece_ids_matching_inventory(self.inventories[color.index()], false)
    }
}

fn piece_ids_matching_inventory(inventory: PieceInventory, used: bool) -> Vec<PieceId> {
    let mut piece_ids = Vec::with_capacity(usize::from(PIECE_COUNT));

    for raw_piece_id in 0..PIECE_COUNT {
        let piece_id = PieceId::try_new(raw_piece_id)
            .unwrap_or_else(|_| unreachable!("piece id in 0..PIECE_COUNT is valid"));

        if inventory.is_used(piece_id) == used {
            piece_ids.push(piece_id);
        }
    }

    piece_ids
}

const LAST_PIECE_SLOT_BITS: u32 = 5;
const LAST_PIECE_SLOT_MASK: u32 = (1u32 << LAST_PIECE_SLOT_BITS) - 1;
const LAST_PIECE_PACKED_MASK: u32 = (1u32 << (LAST_PIECE_SLOT_BITS * 4)) - 1;

const fn last_piece_shift(color: PlayerColor) -> u32 {
    let slot = match color {
        PlayerColor::Blue => 0,
        PlayerColor::Yellow => 1,
        PlayerColor::Red => 2,
        PlayerColor::Green => 3,
    };

    slot * LAST_PIECE_SLOT_BITS
}
