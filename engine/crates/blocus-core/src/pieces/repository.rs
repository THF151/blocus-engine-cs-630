//! Canonical official Blokus piece repository.

use crate::{InputError, OrientationId, PIECE_COUNT, PieceId};
use std::sync::OnceLock;

use super::shape::{Flip, Rotation, ShapeBitmap};

/// Maximum number of unique orientations produced by rotations and one mirror.
pub const MAX_UNIQUE_ORIENTATIONS: usize = 8;

/// Immutable official Blokus piece repository.
///
/// The repository owns the canonical pieces and their precomputed orientations.
/// It is initialized once and shared by all engines.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct PieceRepository {
    pieces: [CanonicalPiece; PIECE_COUNT as usize],
}

impl PieceRepository {
    fn new() -> Self {
        Self {
            pieces: [
                piece(0, "I1", &[(0, 0)]),
                piece(1, "I2", &[(0, 0), (0, 1)]),
                piece(2, "I3", &[(0, 0), (0, 1), (0, 2)]),
                piece(3, "V3", &[(0, 0), (1, 0), (1, 1)]),
                piece(4, "I4", &[(0, 0), (0, 1), (0, 2), (0, 3)]),
                piece(5, "O4", &[(0, 0), (0, 1), (1, 0), (1, 1)]),
                piece(6, "T4", &[(0, 0), (0, 1), (0, 2), (1, 1)]),
                piece(7, "L4", &[(0, 0), (1, 0), (2, 0), (2, 1)]),
                piece(8, "Z4", &[(0, 0), (0, 1), (1, 1), (1, 2)]),
                piece(9, "F5", &[(0, 1), (1, 0), (1, 1), (1, 2), (2, 2)]),
                piece(10, "I5", &[(0, 0), (0, 1), (0, 2), (0, 3), (0, 4)]),
                piece(11, "L5", &[(0, 0), (1, 0), (2, 0), (3, 0), (3, 1)]),
                piece(12, "N5", &[(0, 1), (1, 1), (2, 0), (2, 1), (3, 0)]),
                piece(13, "P5", &[(0, 0), (0, 1), (1, 0), (1, 1), (2, 0)]),
                piece(14, "T5", &[(0, 0), (0, 1), (0, 2), (1, 1), (2, 1)]),
                piece(15, "U5", &[(0, 0), (0, 2), (1, 0), (1, 1), (1, 2)]),
                piece(16, "V5", &[(0, 0), (1, 0), (2, 0), (2, 1), (2, 2)]),
                piece(17, "W5", &[(0, 0), (1, 0), (1, 1), (2, 1), (2, 2)]),
                piece(18, "X5", &[(0, 1), (1, 0), (1, 1), (1, 2), (2, 1)]),
                piece(19, "Y5", &[(0, 0), (1, 0), (2, 0), (3, 0), (2, 1)]),
                piece(20, "Z5", &[(0, 0), (0, 1), (1, 1), (2, 1), (2, 2)]),
            ],
        }
    }

    /// Returns all official pieces.
    #[must_use]
    pub const fn pieces(&self) -> &[CanonicalPiece; PIECE_COUNT as usize] {
        &self.pieces
    }

    /// Returns one official piece by id.
    #[must_use]
    pub const fn piece(&self, piece_id: PieceId) -> &CanonicalPiece {
        &self.pieces[piece_id.as_u8() as usize]
    }
}

/// A precomputed unique orientation for a canonical piece.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct PieceOrientation {
    id: OrientationId,
    shape: ShapeBitmap,
}

impl PieceOrientation {
    /// Creates a piece orientation.
    #[must_use]
    pub const fn new(id: OrientationId, shape: ShapeBitmap) -> Self {
        Self { id, shape }
    }

    /// Returns the stable orientation identifier for this piece.
    #[must_use]
    pub const fn id(self) -> OrientationId {
        self.id
    }

    /// Returns the normalized shape for this orientation.
    #[must_use]
    pub const fn shape(self) -> ShapeBitmap {
        self.shape
    }
}

/// One official Blokus piece with all unique orientations precomputed.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct CanonicalPiece {
    id: PieceId,
    name: &'static str,
    base_shape: ShapeBitmap,
    orientations: [Option<PieceOrientation>; MAX_UNIQUE_ORIENTATIONS],
    orientation_count: u8,
}

impl CanonicalPiece {
    /// Creates a canonical piece and precomputes all unique orientations.
    ///
    /// # Errors
    ///
    /// Returns [`InputError::InvalidGameConfig`] if the base cells are not a
    /// valid compact shape.
    pub fn try_new(
        id: PieceId,
        name: &'static str,
        cells: &[(u8, u8)],
    ) -> Result<Self, InputError> {
        let base_shape = ShapeBitmap::from_cells(cells)?;
        let (orientations, orientation_count) = generate_unique_orientations(base_shape);

        Ok(Self {
            id,
            name,
            base_shape,
            orientations,
            orientation_count,
        })
    }

    #[must_use]
    pub const fn id(self) -> PieceId {
        self.id
    }

    #[must_use]
    pub const fn name(self) -> &'static str {
        self.name
    }

    #[must_use]
    pub const fn base_shape(self) -> ShapeBitmap {
        self.base_shape
    }

    #[must_use]
    pub const fn square_count(self) -> u8 {
        self.base_shape.square_count()
    }

    #[must_use]
    pub const fn orientation_count(self) -> u8 {
        self.orientation_count
    }

    #[must_use]
    pub fn orientation(self, id: OrientationId) -> Option<PieceOrientation> {
        let index = usize::from(id.as_u8());

        if index < usize::from(self.orientation_count) {
            self.orientations[index]
        } else {
            None
        }
    }

    #[must_use]
    pub fn orientations(self) -> Vec<PieceOrientation> {
        self.orientations
            .into_iter()
            .take(usize::from(self.orientation_count))
            .flatten()
            .collect()
    }
}

/// Returns the immutable official repository.
#[must_use]
pub fn standard_repository() -> &'static PieceRepository {
    STANDARD_REPOSITORY.get_or_init(PieceRepository::new)
}

/// Returns the immutable official 21-piece slice.
///
/// Orientations are generated once on first repository use and then reused.
#[must_use]
pub fn standard_pieces() -> &'static [CanonicalPiece; PIECE_COUNT as usize] {
    standard_repository().pieces()
}

/// Returns one official piece by id.
#[must_use]
pub fn standard_piece(piece_id: PieceId) -> &'static CanonicalPiece {
    standard_repository().piece(piece_id)
}

static STANDARD_REPOSITORY: OnceLock<PieceRepository> = OnceLock::new();

fn piece(id: u8, name: &'static str, cells: &[(u8, u8)]) -> CanonicalPiece {
    let Ok(piece_id) = PieceId::try_new(id) else {
        panic!("canonical piece id must be valid");
    };

    CanonicalPiece::try_new(piece_id, name, cells)
        .unwrap_or_else(|_| panic!("canonical piece {name} must be a valid shape"))
}

fn generate_unique_orientations(
    base_shape: ShapeBitmap,
) -> ([Option<PieceOrientation>; MAX_UNIQUE_ORIENTATIONS], u8) {
    let mut orientations = [None; MAX_UNIQUE_ORIENTATIONS];
    let mut count = 0usize;

    for flip in Flip::ALL {
        for rotation in Rotation::ALL {
            let transformed = base_shape.transformed(rotation, flip);

            if contains_shape(&orientations, count, transformed) {
                continue;
            }

            let Ok(raw_orientation_id) = u8::try_from(count) else {
                unreachable!("orientation count never exceeds 8");
            };
            let Ok(orientation_id) = OrientationId::try_new(raw_orientation_id) else {
                unreachable!("generated orientation id is always in range");
            };

            orientations[count] = Some(PieceOrientation::new(orientation_id, transformed));
            count += 1;
        }
    }

    let Ok(orientation_count) = u8::try_from(count) else {
        unreachable!("orientation count never exceeds 8");
    };

    (orientations, orientation_count)
}

fn contains_shape(
    orientations: &[Option<PieceOrientation>; MAX_UNIQUE_ORIENTATIONS],
    count: usize,
    shape: ShapeBitmap,
) -> bool {
    let mut index = 0usize;

    while index < count {
        if let Some(orientation) = orientations[index]
            && orientation.shape() == shape
        {
            return true;
        }

        index += 1;
    }

    false
}
