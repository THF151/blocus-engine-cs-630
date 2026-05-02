use blocus_core::{Flip, InputError, Rotation, ShapeBitmap};

fn shape(cells: &[(u8, u8)]) -> ShapeBitmap {
    let Ok(shape) = ShapeBitmap::from_cells(cells) else {
        panic!("shape should be valid");
    };

    shape
}

#[test]
fn shape_bitmap_normalizes_cells_to_top_left_origin() {
    let normalized = shape(&[(2, 3), (3, 3), (3, 4)]);

    assert_eq!(normalized.width(), 2);
    assert_eq!(normalized.height(), 2);
    assert_eq!(normalized.square_count(), 3);

    assert!(normalized.contains(0, 0));
    assert!(normalized.contains(1, 0));
    assert!(normalized.contains(1, 1));
    assert!(!normalized.contains(0, 1));

    assert_eq!(normalized.cells(), vec![(0, 0), (1, 0), (1, 1)]);
}

#[test]
fn shape_bitmap_counts_squares_and_preserves_bounds() {
    let piece = shape(&[(0, 0), (0, 1), (0, 2), (1, 1)]);

    assert_eq!(piece.width(), 3);
    assert_eq!(piece.height(), 2);
    assert_eq!(piece.square_count(), 4);
    assert_eq!(piece.cells().len(), 4);
}

#[test]
fn shape_bitmap_rejects_empty_shapes() {
    assert_eq!(
        ShapeBitmap::from_cells(&[]),
        Err(InputError::InvalidGameConfig)
    );
}

#[test]
fn shape_bitmap_rejects_duplicate_cells() {
    assert_eq!(
        ShapeBitmap::from_cells(&[(0, 0), (0, 0)]),
        Err(InputError::InvalidGameConfig)
    );
}

#[test]
fn shape_bitmap_rejects_cells_outside_local_five_by_five_grid() {
    assert_eq!(
        ShapeBitmap::from_cells(&[(0, 0), (5, 0)]),
        Err(InputError::InvalidGameConfig)
    );
    assert_eq!(
        ShapeBitmap::from_cells(&[(0, 0), (0, 5)]),
        Err(InputError::InvalidGameConfig)
    );
}

#[test]
fn shape_bitmap_rejects_more_than_five_cells() {
    assert_eq!(
        ShapeBitmap::from_cells(&[(0, 0), (0, 1), (0, 2), (0, 3), (0, 4), (1, 0)]),
        Err(InputError::InvalidGameConfig)
    );
}

#[test]
fn shape_bitmap_from_raw_mask_normalizes_shape() {
    let original = shape(&[(1, 1), (1, 2), (2, 1)]);
    let Ok(from_mask) = ShapeBitmap::from_raw_mask(original.cell_mask() << 6) else {
        panic!("shifted raw mask should still be inside local grid");
    };

    assert_eq!(from_mask, original);
}

#[test]
fn rotation_90_transforms_asymmetric_shape() {
    let piece = shape(&[(0, 0), (1, 0), (1, 1)]);
    let rotated = piece.transformed(Rotation::Deg90, Flip::None);

    assert_eq!(rotated.cells(), vec![(0, 0), (0, 1), (1, 0)]);
}

#[test]
fn rotation_180_transforms_asymmetric_shape() {
    let piece = shape(&[(0, 0), (1, 0), (1, 1)]);
    let rotated = piece.transformed(Rotation::Deg180, Flip::None);

    assert_eq!(rotated.cells(), vec![(0, 0), (0, 1), (1, 1)]);
}

#[test]
fn rotation_270_transforms_asymmetric_shape() {
    let piece = shape(&[(0, 0), (1, 0), (1, 1)]);
    let rotated = piece.transformed(Rotation::Deg270, Flip::None);

    assert_eq!(rotated.cells(), vec![(0, 1), (1, 0), (1, 1)]);
}

#[test]
fn horizontal_flip_transforms_asymmetric_shape() {
    let piece = shape(&[(0, 0), (1, 0), (1, 1)]);
    let flipped = piece.transformed(Rotation::Deg0, Flip::Horizontal);

    assert_eq!(flipped.cells(), vec![(0, 1), (1, 0), (1, 1)]);
}

#[test]
fn symmetric_square_keeps_same_shape_under_all_transforms() {
    let square = shape(&[(0, 0), (0, 1), (1, 0), (1, 1)]);

    for flip in Flip::ALL {
        for rotation in Rotation::ALL {
            assert_eq!(square.transformed(rotation, flip), square);
        }
    }
}

#[test]
fn shape_bitmap_is_copy_eq_hash_and_debug() {
    let first = shape(&[(0, 0), (0, 1), (1, 0)]);
    let duplicate = first;
    let copied = first;
    let different = shape(&[(0, 0), (1, 0), (1, 1)]);

    assert_eq!(first, copied);
    assert_eq!(first, duplicate);
    assert_ne!(first, different);
    assert!(format!("{first:?}").contains("ShapeBitmap"));
}
