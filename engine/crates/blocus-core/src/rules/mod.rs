//! Blokus rule validation.

pub(crate) mod placement;

pub use placement::{Placement, build_placement, validate_place_command};
