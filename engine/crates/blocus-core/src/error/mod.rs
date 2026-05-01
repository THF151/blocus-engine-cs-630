//! Typed domain error hierarchy.

use std::fmt;

macro_rules! define_error_kind {
    (
        $(#[$enum_meta:meta])*
        pub enum $name:ident {
            category: $category:literal,
            $(
                $(#[$variant_meta:meta])*
                $variant:ident => {
                    code: $code:literal,
                    message: $message:literal $(,)?
                }
            ),+ $(,)?
        }
    ) => {
        $(#[$enum_meta])*
        #[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
        #[non_exhaustive]
        pub enum $name {
            $(
                $(#[$variant_meta])*
                $variant,
            )+
        }

        impl $name {
            /// Stable category for this error kind.
            pub const CATEGORY: &'static str = $category;

            /// Stable error category.
            #[must_use]
            pub const fn category(self) -> &'static str {
                Self::CATEGORY
            }

            /// Stable machine-readable error code.
            #[must_use]
            pub const fn code(self) -> &'static str {
                match self {
                    $(
                        Self::$variant => $code,
                    )+
                }
            }

            /// Human-readable error message.
            #[must_use]
            pub const fn message(self) -> &'static str {
                match self {
                    $(
                        Self::$variant => $message,
                    )+
                }
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(formatter, "{}: {}", self.code(), self.message())
            }
        }

        impl std::error::Error for $name {}
    };
}

macro_rules! impl_domain_error_from {
    ($source:ty, $variant:ident) => {
        impl From<$source> for DomainError {
            fn from(value: $source) -> Self {
                Self::$variant(value)
            }
        }
    };
}

/// Unified domain error returned by public engine APIs.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum DomainError {
    /// Recoverable violation of Blokus rules.
    RuleViolation(RuleViolation),
    /// Malformed or inconsistent caller input.
    InputError(InputError),
    /// Internal engine failure or corrupted state.
    EngineError(EngineError),
}

impl DomainError {
    /// Stable machine-readable error code.
    #[must_use]
    pub const fn code(&self) -> &'static str {
        match self {
            Self::RuleViolation(error) => error.code(),
            Self::InputError(error) => error.code(),
            Self::EngineError(error) => error.code(),
        }
    }

    /// Human-readable error message.
    #[must_use]
    pub const fn message(&self) -> &'static str {
        match self {
            Self::RuleViolation(error) => error.message(),
            Self::InputError(error) => error.message(),
            Self::EngineError(error) => error.message(),
        }
    }

    /// Stable error category.
    #[must_use]
    pub const fn category(&self) -> &'static str {
        match self {
            Self::RuleViolation(error) => error.category(),
            Self::InputError(error) => error.category(),
            Self::EngineError(error) => error.category(),
        }
    }
}

impl_domain_error_from!(RuleViolation, RuleViolation);
impl_domain_error_from!(InputError, InputError);
impl_domain_error_from!(EngineError, EngineError);

impl fmt::Display for DomainError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}: {}", self.code(), self.message())
    }
}

impl std::error::Error for DomainError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::RuleViolation(error) => Some(error),
            Self::InputError(error) => Some(error),
            Self::EngineError(error) => Some(error),
        }
    }
}

define_error_kind! {
    /// Recoverable violation of Blokus rules.
    pub enum RuleViolation {
        category: "rule_violation",

        /// It is not this color's turn.
        WrongPlayerTurn => {
            code: "WrongPlayerTurn",
            message: "it is not this color's turn",
        },

        /// The player does not control the submitted color.
        PlayerDoesNotControlColor => {
            code: "PlayerDoesNotControlColor",
            message: "player does not control this color",
        },

        /// The selected piece has already been used by this color.
        PieceAlreadyUsed => {
            code: "PieceAlreadyUsed",
            message: "piece has already been used",
        },

        /// The placement would touch non-playable board space.
        OutOfBounds => {
            code: "OutOfBounds",
            message: "placement is outside the playable board",
        },

        /// The placement overlaps an occupied board cell.
        Overlap => {
            code: "Overlap",
            message: "placement overlaps an occupied cell",
        },

        /// The placement is missing required same-color diagonal contact.
        MissingCornerContact => {
            code: "MissingCornerContact",
            message: "placement must touch same-color piece by corner",
        },

        /// The placement has forbidden same-color edge contact.
        IllegalEdgeContact => {
            code: "IllegalEdgeContact",
            message: "placement must not touch same-color piece by edge",
        },

        /// The game has already finished.
        GameAlreadyFinished => {
            code: "GameAlreadyFinished",
            message: "the game has already finished",
        },

        /// A pass was requested while at least one legal move exists.
        PassNotAllowedBecauseMoveExists => {
            code: "PassNotAllowedBecauseMoveExists",
            message: "pass is not allowed while a legal move exists",
        },

        /// Final scoring was requested before the game finished.
        GameNotFinished => {
            code: "GameNotFinished",
            message: "the game is not finished",
        },
    }
}

define_error_kind! {
    /// Malformed or inconsistent caller input.
    pub enum InputError {
        category: "input_error",

        /// Command game identifier does not match state game identifier.
        GameIdMismatch => {
            code: "GameIdMismatch",
            message: "command game ID does not match state game ID",
        },

        /// Referenced player is unknown in the current game.
        UnknownPlayer => {
            code: "UnknownPlayer",
            message: "unknown player",
        },

        /// Referenced piece does not exist.
        UnknownPiece => {
            code: "UnknownPiece",
            message: "unknown piece",
        },

        /// Referenced orientation does not exist for the piece.
        UnknownOrientation => {
            code: "UnknownOrientation",
            message: "unknown orientation",
        },

        /// Board index or row/column coordinate is invalid.
        InvalidBoardIndex => {
            code: "InvalidBoardIndex",
            message: "invalid board index",
        },

        /// Game configuration is invalid.
        InvalidGameConfig => {
            code: "InvalidGameConfig",
            message: "invalid game configuration",
        },

        /// State version is invalid for the requested operation.
        InvalidStateVersion => {
            code: "InvalidStateVersion",
            message: "invalid state version",
        },
    }
}

define_error_kind! {
    /// Internal engine failure or corrupted state.
    pub enum EngineError {
        category: "engine_error",

        /// State violates internal invariants.
        CorruptedState => {
            code: "CorruptedState",
            message: "corrupted game state",
        },

        /// Engine reached an impossible invariant violation.
        InvariantViolation => {
            code: "InvariantViolation",
            message: "engine invariant violation",
        },

        /// Standard piece repository could not be initialized.
        RepositoryInitializationFailed => {
            code: "RepositoryInitializationFailed",
            message: "piece repository initialization failed",
        },
    }
}
