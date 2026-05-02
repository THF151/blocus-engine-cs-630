use blocus_core::{PIECE_COUNT, PieceId, standard_piece, standard_pieces};

fn piece_id(value: u8) -> PieceId {
    let Ok(piece_id) = PieceId::try_new(value) else {
        panic!("piece id should be valid");
    };

    piece_id
}

#[test]
fn standard_repository_contains_exactly_twenty_one_pieces() {
    let pieces = standard_pieces();

    assert_eq!(pieces.len(), usize::from(PIECE_COUNT));

    for expected_id in 0..PIECE_COUNT {
        assert_eq!(pieces[usize::from(expected_id)].id(), piece_id(expected_id));
    }
}

#[test]
fn standard_repository_has_official_size_distribution() {
    let mut counts = [0usize; 6];

    for piece in standard_pieces() {
        counts[usize::from(piece.square_count())] += 1;
    }

    assert_eq!(counts[1], 1);
    assert_eq!(counts[2], 1);
    assert_eq!(counts[3], 2);
    assert_eq!(counts[4], 5);
    assert_eq!(counts[5], 12);
}

#[test]
fn standard_repository_piece_names_are_stable() {
    let names = standard_pieces()
        .iter()
        .map(|piece| piece.name())
        .collect::<Vec<_>>();

    assert_eq!(
        names,
        vec![
            "I1", "I2", "I3", "V3", "I4", "O4", "T4", "L4", "Z4", "F5", "I5", "L5", "N5", "P5",
            "T5", "U5", "V5", "W5", "X5", "Y5", "Z5",
        ]
    );
}

#[test]
fn standard_piece_lookup_returns_piece_by_id_without_recalculation() {
    for raw_id in 0..PIECE_COUNT {
        let by_slice = standard_pieces()[usize::from(raw_id)];
        let by_lookup = *standard_piece(piece_id(raw_id));

        assert_eq!(by_lookup, by_slice);
    }

    assert!(std::ptr::eq(standard_pieces(), standard_pieces()));
}

#[test]
fn generated_orientation_counts_are_stable_and_deduplicated() {
    let expected_counts = [
        1, 2, 2, 4, 2, 1, 4, 8, 4, 8, 2, 8, 8, 8, 4, 4, 4, 4, 1, 8, 4,
    ];

    for (raw_id, expected_count) in expected_counts.into_iter().enumerate() {
        let Ok(raw_id) = u8::try_from(raw_id) else {
            panic!("piece id index should fit in u8");
        };

        let piece = standard_piece(piece_id(raw_id));

        assert_eq!(
            piece.orientation_count(),
            expected_count,
            "unexpected orientation count for piece {}",
            piece.name()
        );
    }
}

#[test]
fn generated_orientations_have_stable_local_ids_and_matching_square_counts() {
    for piece in standard_pieces() {
        let orientations = piece.orientations();

        assert_eq!(orientations.len(), usize::from(piece.orientation_count()));

        for (index, orientation) in orientations.iter().enumerate() {
            let Ok(raw_orientation_id) = u8::try_from(index) else {
                panic!("orientation index should fit in u8");
            };

            assert_eq!(orientation.id().as_u8(), raw_orientation_id);
            assert_eq!(orientation.shape().square_count(), piece.square_count());
        }
    }
}

#[test]
fn generated_orientations_do_not_contain_duplicates_per_piece() {
    for piece in standard_pieces() {
        let orientations = piece.orientations();

        for left in 0..orientations.len() {
            for right in (left + 1)..orientations.len() {
                assert_ne!(
                    orientations[left].shape(),
                    orientations[right].shape(),
                    "piece {} has duplicate orientations at {left} and {right}",
                    piece.name()
                );
            }
        }
    }
}

#[test]
fn orientation_lookup_rejects_ids_outside_unique_count() {
    for piece in standard_pieces() {
        for raw_id in piece.orientation_count()..8 {
            let Ok(orientation_id) = blocus_core::OrientationId::try_new(raw_id) else {
                panic!("orientation id below 8 should be valid");
            };

            assert_eq!(piece.orientation(orientation_id), None);
        }
    }
}

#[test]
fn base_shapes_are_normalized_and_have_expected_square_counts() {
    for piece in standard_pieces() {
        let base = piece.base_shape();

        assert!(base.width() >= 1);
        assert!(base.height() >= 1);
        assert_eq!(base.square_count(), piece.square_count());
        assert_eq!(base.cells().len(), usize::from(piece.square_count()));
    }
}
