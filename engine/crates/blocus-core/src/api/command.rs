//! Command DTOs accepted by the engine.

use crate::{BoardIndex, CommandId, GameId, OrientationId, PieceId, PlayerColor, PlayerId};

/// Command submitted to the engine.
///
/// A Blokus turn is one of two actions: place a piece or pass.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum Command {
    /// Place a piece.
    Place(PlaceCommand),
    /// Pass the current turn.
    Pass(PassCommand),
}

/// Place-piece command.
///
/// The Python adapter may accept row/column input, but the Rust core receives
/// a validated [`BoardIndex`].
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct PlaceCommand {
    /// Unique command identifier.
    pub command_id: CommandId,
    /// Target game identifier.
    pub game_id: GameId,
    /// Submitting player.
    pub player_id: PlayerId,
    /// Color being played.
    pub color: PlayerColor,
    /// Piece to place.
    pub piece_id: PieceId,
    /// Precomputed orientation to place.
    pub orientation_id: OrientationId,
    /// Anchor cell using padded-row board indexing.
    pub anchor: BoardIndex,
}

/// Pass-turn command.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct PassCommand {
    /// Unique command identifier.
    pub command_id: CommandId,
    /// Target game identifier.
    pub game_id: GameId,
    /// Submitting player.
    pub player_id: PlayerId,
    /// Color being played.
    pub color: PlayerColor,
}
