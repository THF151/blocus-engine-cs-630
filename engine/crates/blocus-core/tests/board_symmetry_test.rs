use blocus_core::{BoardIndex, BoardMask, BoardSymmetry};

fn index(row: u8, col: u8) -> BoardIndex {
    BoardIndex::from_row_col(row, col)
        .unwrap_or_else(|_| panic!("row {row}, col {col} should be valid"))
}

#[test]
fn board_symmetry_transforms_representative_indices() {
    let source = index(2, 5);

    assert_eq!(BoardSymmetry::Identity.transform_index(source), index(2, 5));
    assert_eq!(
        BoardSymmetry::Rotate90.transform_index(source),
        index(5, 17)
    );
    assert_eq!(
        BoardSymmetry::Rotate180.transform_index(source),
        index(17, 14)
    );
    assert_eq!(
        BoardSymmetry::Rotate270.transform_index(source),
        index(14, 2)
    );
    assert_eq!(
        BoardSymmetry::ReflectHorizontal.transform_index(source),
        index(17, 5)
    );
    assert_eq!(
        BoardSymmetry::ReflectVertical.transform_index(source),
        index(2, 14)
    );
    assert_eq!(
        BoardSymmetry::ReflectMainDiagonal.transform_index(source),
        index(5, 2)
    );
    assert_eq!(
        BoardSymmetry::ReflectAntiDiagonal.transform_index(source),
        index(14, 17)
    );
}

#[test]
fn every_board_symmetry_round_trips_through_inverse() {
    for symmetry in BoardSymmetry::ALL {
        for source in [index(0, 0), index(2, 5), index(19, 18)] {
            let transformed = symmetry.transform_index(source);
            assert_eq!(symmetry.inverse().transform_index(transformed), source);
        }
    }
}

#[test]
fn mask_transform_preserves_count_and_supports_round_trip() {
    let mask = BoardMask::from_index(index(0, 0))
        .union(BoardMask::from_index(index(2, 5)))
        .union(BoardMask::from_index(index(19, 18)));

    for symmetry in BoardSymmetry::ALL {
        let transformed = mask.transformed(symmetry);
        assert_eq!(transformed.count(), mask.count());
        assert_eq!(transformed.transformed(symmetry.inverse()), mask);
    }
}
