//! Deterministic position hashing.
//!
//! `GameState::hash` is a Zobrist-style position hash: it identifies the board
//! position and rule context, not replay identity. It intentionally excludes
//! `GameState::hash`, `game_id`, `schema_version`, command ids, and
//! `version`, so equivalent positions reached through different games or
//! version histories share the same hash.

use crate::{
    BOARD_SIZE, BoardMask, GameMode, GameState, GameStatus, LastPieceByColor, PIECE_COUNT,
    PLAYER_COLOR_COUNT, PieceId, PlayerColor, ScoringMode, TurnState, ZobristHash,
};

/// Fixed domain seed for position hashing.
pub const HASH_DOMAIN_SEED: u64 = 0x9e37_79b9_7f4a_7c15;

/// Hash salt for occupied board cells.
pub const HASH_BOARD_CELL_SALT: u64 = 0xbb67_ae85_84ca_a73b;

/// Hash salt for piece inventory bits.
pub const HASH_INVENTORY_SALT: u64 = 0x3c6e_f372_fe94_f82b;

/// Hash salt for player/color topology.
pub const HASH_PLAYER_SLOT_SALT: u64 = 0xa54f_f53a_5f1d_36f1;

/// Hash salt for turn-state fields.
pub const HASH_TURN_SALT: u64 = 0x510e_527f_ade6_82d1;

/// Hash salt for last-piece tracking.
pub const HASH_LAST_PIECE_SALT: u64 = 0x1f83_d9ab_fb41_bd6b;

const HASH_MODE_SALT: u64 = 0xc2b2_ae3d_27d4_eb4f;
const HASH_SCORING_SALT: u64 = 0x1656_67b1_9e37_79f9;
const HASH_STATUS_SALT: u64 = 0x85eb_ca6b_27d4_eb2f;
const HASH_TURN_ORDER_SALT: u64 = 0x27d4_eb2f_1656_67c5;

/// Computes the full deterministic position hash for a game state.
///
/// The stored `state.hash` field is deliberately excluded, so this function is
/// stable for verification and round trips.
///
/// # Examples
///
/// ```
/// # use blocus_core::{BlocusEngine, GameConfig, GameId, GameMode, PlayerId, PlayerSlots, ScoringMode, TurnOrder, compute_hash_full};
/// # use uuid::Uuid;
/// # let game_id = GameId::from_uuid(Uuid::from_u128(1));
/// # let p1 = PlayerId::from_uuid(Uuid::from_u128(2));
/// # let p2 = PlayerId::from_uuid(Uuid::from_u128(3));
/// # let slots = PlayerSlots::two_player(p1, p2).unwrap();
/// # let config = GameConfig::try_new(game_id, GameMode::TwoPlayer, ScoringMode::Basic, TurnOrder::OFFICIAL_FIXED, slots).unwrap();
/// # let engine = BlocusEngine::new();
/// let state = engine.initialize_game(config);
/// assert_eq!(state.hash, compute_hash_full(&state));
/// ```
#[must_use]
pub fn compute_hash_full(state: &GameState) -> ZobristHash {
    let mut hash = HASH_DOMAIN_SEED;

    hash ^= game_mode_hash(state.mode);
    hash ^= scoring_mode_hash(state.scoring);
    hash ^= turn_order_hash(state.turn_order);
    hash ^= player_slot_topology_hash(state.player_slots);
    hash ^= board_hash(state);
    hash ^= inventory_hash(state);
    hash ^= last_piece_hash(state.last_piece_by_color);
    hash ^= turn_state_hash(state.turn);
    hash ^= game_status_hash(state.status);

    ZobristHash::new(hash)
}

/// Deterministic hash constant for one occupied board cell.
///
/// `bit_index` is the padded-row board bit index, not dense row-major index.
#[must_use]
pub const fn board_cell_hash(color: PlayerColor, bit_index: u16) -> u64 {
    zobrist_const(
        HASH_BOARD_CELL_SALT,
        ((color_index_u64(color) + 1) << 48) ^ bit_index as u64,
    )
}

/// Deterministic hash constant for one used piece bit in one color inventory.
#[must_use]
pub const fn inventory_piece_hash(color: PlayerColor, piece_id: u8) -> u64 {
    zobrist_const(
        HASH_INVENTORY_SALT,
        ((color_index_u64(color) + 1) << 48) ^ piece_id as u64,
    )
}

/// XORs one newly placed piece into an existing position hash.
#[must_use]
pub fn xor_place_piece(
    hash: ZobristHash,
    color: PlayerColor,
    placement_mask: BoardMask,
    piece_id: PieceId,
) -> ZobristHash {
    let mut next = hash.as_u64();
    next = xor_board_mask_cells(next, color, placement_mask);
    next ^= inventory_piece_hash(color, piece_id.as_u8());
    ZobristHash::new(next)
}

/// XORs out one turn state and XORs in another.
#[must_use]
pub fn xor_turn_transition(
    hash: ZobristHash,
    old_turn: TurnState,
    new_turn: TurnState,
) -> ZobristHash {
    ZobristHash::new(hash.as_u64() ^ turn_state_hash(old_turn) ^ turn_state_hash(new_turn))
}

/// XORs out one game status and XORs in another.
#[must_use]
pub fn xor_status_transition(
    hash: ZobristHash,
    old_status: GameStatus,
    new_status: GameStatus,
) -> ZobristHash {
    ZobristHash::new(hash.as_u64() ^ game_status_hash(old_status) ^ game_status_hash(new_status))
}

/// XORs out one last-piece tracker and XORs in another.
#[must_use]
pub fn xor_last_piece_transition(
    hash: ZobristHash,
    old_last_piece: LastPieceByColor,
    new_last_piece: LastPieceByColor,
) -> ZobristHash {
    ZobristHash::new(
        hash.as_u64() ^ last_piece_hash(old_last_piece) ^ last_piece_hash(new_last_piece),
    )
}

fn board_hash(state: &GameState) -> u64 {
    let mut hash = 0u64;

    for color in PlayerColor::ALL {
        hash = xor_board_mask_cells(hash, color, state.board.occupied(color));
    }

    hash
}

fn xor_board_mask_cells(mut hash: u64, color: PlayerColor, mask: BoardMask) -> u64 {
    for (lane_index, lane_value) in mask.lanes().into_iter().enumerate() {
        let mut remaining = lane_value;

        while remaining != 0 {
            let offset = remaining.trailing_zeros();
            let bit_index = u16::try_from(lane_index * u128::BITS as usize + offset as usize)
                .unwrap_or_else(|_| unreachable!("board bit index always fits in u16"));

            hash ^= board_cell_hash(color, bit_index);
            remaining &= remaining - 1;
        }
    }

    hash
}

fn inventory_hash(state: &GameState) -> u64 {
    let mut hash = 0u64;

    for color in PlayerColor::ALL {
        let used_mask = state.inventories[color.index()].used_mask();

        for raw_piece_id in 0..PIECE_COUNT {
            if used_mask & (1u32 << raw_piece_id) != 0 {
                hash ^= inventory_piece_hash(color, raw_piece_id);
            }
        }
    }

    hash
}

fn last_piece_hash(last_piece_by_color: LastPieceByColor) -> u64 {
    zobrist_const(
        HASH_LAST_PIECE_SALT,
        u64::from(last_piece_by_color.packed()),
    )
}

fn turn_state_hash(turn: TurnState) -> u64 {
    let shared_index = u64::try_from(turn.shared_color_turn_index())
        .unwrap_or_else(|_| unreachable!("turn index should fit in u64"));

    zobrist_const(HASH_TURN_SALT ^ 0x01, color_index_u64(turn.current_color()))
        ^ zobrist_const(HASH_TURN_SALT ^ 0x02, u64::from(turn.passed_mask()))
        ^ zobrist_const(HASH_TURN_SALT ^ 0x03, shared_index)
}

fn game_mode_hash(mode: GameMode) -> u64 {
    let value = match mode {
        GameMode::TwoPlayer => 2,
        GameMode::ThreePlayer => 3,
        GameMode::FourPlayer => 4,
    };

    zobrist_const(HASH_MODE_SALT, value)
}

fn scoring_mode_hash(scoring: ScoringMode) -> u64 {
    let value = match scoring {
        ScoringMode::Basic => 0,
        ScoringMode::Advanced => 1,
    };

    zobrist_const(HASH_SCORING_SALT, value)
}

fn game_status_hash(status: GameStatus) -> u64 {
    let value = match status {
        GameStatus::InProgress => 0,
        GameStatus::Finished => 1,
    };

    zobrist_const(HASH_STATUS_SALT, value)
}

fn turn_order_hash(turn_order: crate::TurnOrder) -> u64 {
    let mut hash = 0u64;

    for (index, color) in turn_order.colors().into_iter().enumerate() {
        let index = u64::try_from(index)
            .unwrap_or_else(|_| unreachable!("turn order index always fits in u64"));
        hash ^= zobrist_const(HASH_TURN_ORDER_SALT ^ (index << 8), color_index_u64(color));
    }

    hash
}

fn player_slot_topology_hash(slots: crate::PlayerSlots) -> u64 {
    let mut hash = 0u64;
    let shared_color = slots.shared_color();

    for color in PlayerColor::ALL {
        let slot_kind = if shared_color == Some(color) {
            2
        } else {
            u64::from(slots.controller_of(color).is_some())
        };
        let color_index = u64::try_from(color.index())
            .unwrap_or_else(|_| unreachable!("player color index always fits in u64"));

        hash ^= zobrist_const(HASH_PLAYER_SLOT_SALT ^ (color_index << 8), slot_kind);
    }

    hash
}

const fn color_index_u64(color: PlayerColor) -> u64 {
    match color {
        PlayerColor::Blue => 0,
        PlayerColor::Yellow => 1,
        PlayerColor::Red => 2,
        PlayerColor::Green => 3,
    }
}

const fn zobrist_const(salt: u64, value: u64) -> u64 {
    splitmix64_const(salt ^ value)
}

const fn splitmix64_const(mut value: u64) -> u64 {
    value = value.wrapping_add(0x9e37_79b9_7f4a_7c15);
    value = (value ^ (value >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    value = (value ^ (value >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    value ^ (value >> 31)
}

const _: () = {
    assert!(PLAYER_COLOR_COUNT == 4);
    assert!(BOARD_SIZE == 20);
};
