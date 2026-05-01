//! Domain identifiers.

use core::fmt;
use uuid::Uuid;

/// Identifier for a game.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct GameId(Uuid);

impl GameId {
    /// Creates a game identifier from a UUID.
    #[must_use]
    pub const fn from_uuid(value: Uuid) -> Self {
        Self(value)
    }

    /// Returns the underlying UUID.
    #[must_use]
    pub const fn as_uuid(self) -> Uuid {
        self.0
    }
}

impl From<Uuid> for GameId {
    fn from(value: Uuid) -> Self {
        Self::from_uuid(value)
    }
}

impl From<GameId> for Uuid {
    fn from(value: GameId) -> Self {
        value.0
    }
}

impl fmt::Display for GameId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

/// Identifier for a human or AI player.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct PlayerId(Uuid);

impl PlayerId {
    /// Creates a player identifier from a UUID.
    #[must_use]
    pub const fn from_uuid(value: Uuid) -> Self {
        Self(value)
    }

    /// Returns the underlying UUID.
    #[must_use]
    pub const fn as_uuid(self) -> Uuid {
        self.0
    }
}

impl From<Uuid> for PlayerId {
    fn from(value: Uuid) -> Self {
        Self::from_uuid(value)
    }
}

impl From<PlayerId> for Uuid {
    fn from(value: PlayerId) -> Self {
        value.0
    }
}

impl fmt::Display for PlayerId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

/// Identifier for a submitted command.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct CommandId(Uuid);

impl CommandId {
    /// Creates a command identifier from a UUID.
    #[must_use]
    pub const fn from_uuid(value: Uuid) -> Self {
        Self(value)
    }

    /// Returns the underlying UUID.
    #[must_use]
    pub const fn as_uuid(self) -> Uuid {
        self.0
    }
}

impl From<Uuid> for CommandId {
    fn from(value: Uuid) -> Self {
        Self::from_uuid(value)
    }
}

impl From<CommandId> for Uuid {
    fn from(value: CommandId) -> Self {
        value.0
    }
}

impl fmt::Display for CommandId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

/// Number of official Blokus pieces per color.
pub const PIECE_COUNT: u8 = 21;

/// Identifier for one of the 21 canonical Blokus pieces.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct PieceId(u8);

impl PieceId {
    /// Creates a piece identifier if the value is in range.
    ///
    /// # Errors
    ///
    /// Returns [`SmallIdError::OutOfRange`] if `value >= 21`.
    pub const fn try_new(value: u8) -> Result<Self, SmallIdError> {
        if value < PIECE_COUNT {
            Ok(Self(value))
        } else {
            Err(SmallIdError::OutOfRange {
                value,
                upper_exclusive: PIECE_COUNT,
            })
        }
    }

    /// Returns the compact integer representation.
    #[must_use]
    pub const fn as_u8(self) -> u8 {
        self.0
    }

    /// Returns the bit mask for this piece in a 32-bit piece inventory.
    #[must_use]
    pub const fn inventory_bit(self) -> u32 {
        1u32 << self.0
    }
}

impl TryFrom<u8> for PieceId {
    type Error = SmallIdError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Self::try_new(value)
    }
}

impl From<PieceId> for u8 {
    fn from(value: PieceId) -> Self {
        value.0
    }
}

impl fmt::Display for PieceId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

/// Maximum number of unique orientations for a Blokus piece.
pub const MAX_ORIENTATION_COUNT: u8 = 8;

/// Identifier for a precomputed unique piece orientation.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct OrientationId(u8);

impl OrientationId {
    /// Creates an orientation identifier if the value is in range.
    ///
    /// # Errors
    ///
    /// Returns [`SmallIdError::OutOfRange`] if `value >= 8`.
    pub const fn try_new(value: u8) -> Result<Self, SmallIdError> {
        if value < MAX_ORIENTATION_COUNT {
            Ok(Self(value))
        } else {
            Err(SmallIdError::OutOfRange {
                value,
                upper_exclusive: MAX_ORIENTATION_COUNT,
            })
        }
    }

    /// Returns the compact integer representation.
    #[must_use]
    pub const fn as_u8(self) -> u8 {
        self.0
    }
}

impl TryFrom<u8> for OrientationId {
    type Error = SmallIdError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Self::try_new(value)
    }
}

impl From<OrientationId> for u8 {
    fn from(value: OrientationId) -> Self {
        value.0
    }
}

impl fmt::Display for OrientationId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

/// Monotonic state version used by adapters for optimistic concurrency.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct StateVersion(u64);

impl StateVersion {
    /// Initial state version.
    pub const INITIAL: Self = Self(0);

    /// Creates a state version from a raw integer.
    #[must_use]
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    /// Returns the raw integer representation.
    #[must_use]
    pub const fn as_u64(self) -> u64 {
        self.0
    }

    /// Returns the next monotonic state version if it does not overflow.
    #[must_use]
    pub const fn checked_next(self) -> Option<Self> {
        match self.0.checked_add(1) {
            Some(value) => Some(Self(value)),
            None => None,
        }
    }

    /// Returns the next state version, saturating at `u64::MAX`.
    #[must_use]
    pub const fn saturating_next(self) -> Self {
        Self(self.0.saturating_add(1))
    }
}

impl From<StateVersion> for u64 {
    fn from(value: StateVersion) -> Self {
        value.0
    }
}

impl fmt::Display for StateVersion {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

/// Semantic position hash.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct ZobristHash(u64);

impl ZobristHash {
    /// Empty hash value used before semantic hashing is implemented.
    pub const ZERO: Self = Self(0);

    /// Creates a hash from a raw integer.
    #[must_use]
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    /// Returns the raw integer representation.
    #[must_use]
    pub const fn as_u64(self) -> u64 {
        self.0
    }
}

impl From<ZobristHash> for u64 {
    fn from(value: ZobristHash) -> Self {
        value.0
    }
}

impl fmt::Display for ZobristHash {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{:016x}", self.0)
    }
}

/// Error returned when constructing a compact bounded identifier.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum SmallIdError {
    /// Value was outside the valid range.
    OutOfRange {
        /// Provided value.
        value: u8,
        /// Exclusive upper bound.
        upper_exclusive: u8,
    },
}

impl fmt::Display for SmallIdError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::OutOfRange {
                value,
                upper_exclusive,
            } => write!(
                formatter,
                "value {value} is out of range; expected value < {upper_exclusive}"
            ),
        }
    }
}
