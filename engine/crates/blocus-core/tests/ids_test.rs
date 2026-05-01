use blocus_core::{
    CommandId, GameId, MAX_ORIENTATION_COUNT, OrientationId, PIECE_COUNT, PieceId, PlayerId,
    SmallIdError, StateVersion, ZobristHash,
};
use std::collections::HashSet;
use uuid::Uuid;

fn uuid(value: u128) -> Uuid {
    Uuid::from_u128(value)
}

macro_rules! assert_uuid_id_round_trip {
    ($id_ty:ty, $value:expr) => {{
        let raw = uuid($value);
        let id = <$id_ty>::from_uuid(raw);

        assert_eq!(id.as_uuid(), raw);

        let from_uuid = <$id_ty>::from(raw);
        let back_to_uuid: Uuid = from_uuid.into();

        assert_eq!(back_to_uuid, raw);
        assert_eq!(id.to_string(), raw.to_string());
    }};
}

macro_rules! assert_uuid_id_copy_eq_hash_and_order {
    ($id_ty:ty) => {{
        let first = <$id_ty>::from_uuid(uuid(1));
        let duplicate = <$id_ty>::from_uuid(uuid(1));
        let second = <$id_ty>::from_uuid(uuid(2));

        let copied = first;

        assert_eq!(first, copied);
        assert_eq!(first, duplicate);
        assert_ne!(first, second);
        assert!(first < second);

        let mut ids = HashSet::new();
        ids.insert(first);
        ids.insert(duplicate);
        ids.insert(second);

        assert_eq!(ids.len(), 2);
        assert!(ids.contains(&<$id_ty>::from_uuid(uuid(1))));
        assert!(ids.contains(&<$id_ty>::from_uuid(uuid(2))));
    }};
}

#[test]
fn uuid_backed_ids_round_trip_uuid() {
    assert_uuid_id_round_trip!(GameId, 1);
    assert_uuid_id_round_trip!(PlayerId, 2);
    assert_uuid_id_round_trip!(CommandId, 3);
}

#[test]
fn uuid_backed_ids_are_copy_comparable_ordered_and_hashable() {
    assert_uuid_id_copy_eq_hash_and_order!(GameId);
    assert_uuid_id_copy_eq_hash_and_order!(PlayerId);
    assert_uuid_id_copy_eq_hash_and_order!(CommandId);
}

#[test]
fn piece_count_matches_official_blokus_piece_count() {
    assert_eq!(PIECE_COUNT, 21);
}

#[test]
fn piece_id_accepts_exactly_values_below_piece_count() {
    for value in 0..=u8::MAX {
        let result = PieceId::try_new(value);

        if value < PIECE_COUNT {
            let Ok(piece) = result else {
                panic!("value {value} below PIECE_COUNT should be a valid PieceId");
            };

            assert_eq!(piece.as_u8(), value);
            assert_eq!(u8::from(piece), value);
            assert_eq!(piece.to_string(), value.to_string());
            assert_eq!(piece.inventory_bit(), 1u32 << value);
            assert_eq!(PieceId::try_from(value), Ok(piece));
        } else {
            let expected_error = SmallIdError::OutOfRange {
                value,
                upper_exclusive: PIECE_COUNT,
            };

            assert_eq!(result, Err(expected_error));
            assert_eq!(PieceId::try_from(value), Err(expected_error));
        }
    }
}

#[test]
fn piece_id_supports_boundary_values() {
    let Ok(first) = PieceId::try_new(0) else {
        panic!("piece id 0 should be valid");
    };

    let Ok(last) = PieceId::try_new(PIECE_COUNT - 1) else {
        panic!("last official piece id should be valid");
    };

    assert_eq!(first.as_u8(), 0);
    assert_eq!(last.as_u8(), 20);

    assert_eq!(first.to_string(), "0");
    assert_eq!(last.to_string(), "20");

    assert_eq!(first.inventory_bit(), 1);
    assert_eq!(last.inventory_bit(), 1u32 << 20);

    assert_eq!(
        PieceId::try_new(PIECE_COUNT),
        Err(SmallIdError::OutOfRange {
            value: PIECE_COUNT,
            upper_exclusive: PIECE_COUNT,
        })
    );
}

#[test]
fn piece_id_is_copy_comparable_ordered_and_hashable() {
    let Ok(zero) = PieceId::try_new(0) else {
        panic!("piece id 0 should be valid");
    };

    let Ok(duplicate_zero) = PieceId::try_new(0) else {
        panic!("piece id 0 should be valid");
    };

    let Ok(one) = PieceId::try_new(1) else {
        panic!("piece id 1 should be valid");
    };

    let copied = zero;

    assert_eq!(zero, copied);
    assert_eq!(zero, duplicate_zero);
    assert_ne!(zero, one);
    assert!(zero < one);

    let mut pieces = HashSet::new();
    pieces.insert(zero);
    pieces.insert(duplicate_zero);
    pieces.insert(one);

    assert_eq!(pieces.len(), 2);

    let Ok(piece_zero) = PieceId::try_new(0) else {
        panic!("piece id 0 should be valid");
    };

    let Ok(piece_one) = PieceId::try_new(1) else {
        panic!("piece id 1 should be valid");
    };

    assert!(pieces.contains(&piece_zero));
    assert!(pieces.contains(&piece_one));
}

#[test]
fn max_orientation_count_matches_dihedral_orientation_upper_bound() {
    assert_eq!(MAX_ORIENTATION_COUNT, 8);
}

#[test]
fn orientation_id_accepts_exactly_values_below_max_orientation_count() {
    for value in 0..=u8::MAX {
        let result = OrientationId::try_new(value);

        if value < MAX_ORIENTATION_COUNT {
            let Ok(orientation) = result else {
                panic!("value {value} below MAX_ORIENTATION_COUNT should be a valid OrientationId");
            };

            assert_eq!(orientation.as_u8(), value);
            assert_eq!(u8::from(orientation), value);
            assert_eq!(orientation.to_string(), value.to_string());
            assert_eq!(OrientationId::try_from(value), Ok(orientation));
        } else {
            let expected_error = SmallIdError::OutOfRange {
                value,
                upper_exclusive: MAX_ORIENTATION_COUNT,
            };

            assert_eq!(result, Err(expected_error));
            assert_eq!(OrientationId::try_from(value), Err(expected_error));
        }
    }
}

#[test]
fn orientation_id_supports_boundary_values() {
    let Ok(first) = OrientationId::try_new(0) else {
        panic!("orientation id 0 should be valid");
    };

    let Ok(last) = OrientationId::try_new(MAX_ORIENTATION_COUNT - 1) else {
        panic!("last orientation id should be valid");
    };

    assert_eq!(first.as_u8(), 0);
    assert_eq!(last.as_u8(), 7);

    assert_eq!(first.to_string(), "0");
    assert_eq!(last.to_string(), "7");

    assert_eq!(
        OrientationId::try_new(MAX_ORIENTATION_COUNT),
        Err(SmallIdError::OutOfRange {
            value: MAX_ORIENTATION_COUNT,
            upper_exclusive: MAX_ORIENTATION_COUNT,
        })
    );
}

#[test]
fn orientation_id_is_copy_comparable_ordered_and_hashable() {
    let Ok(zero) = OrientationId::try_new(0) else {
        panic!("orientation id 0 should be valid");
    };

    let Ok(duplicate_zero) = OrientationId::try_new(0) else {
        panic!("orientation id 0 should be valid");
    };

    let Ok(one) = OrientationId::try_new(1) else {
        panic!("orientation id 1 should be valid");
    };

    let copied = zero;

    assert_eq!(zero, copied);
    assert_eq!(zero, duplicate_zero);
    assert_ne!(zero, one);
    assert!(zero < one);

    let mut orientations = HashSet::new();
    orientations.insert(zero);
    orientations.insert(duplicate_zero);
    orientations.insert(one);

    assert_eq!(orientations.len(), 2);

    let Ok(orientation_zero) = OrientationId::try_new(0) else {
        panic!("orientation id 0 should be valid");
    };

    let Ok(orientation_one) = OrientationId::try_new(1) else {
        panic!("orientation id 1 should be valid");
    };

    assert!(orientations.contains(&orientation_zero));
    assert!(orientations.contains(&orientation_one));
}

#[test]
fn state_version_initial_and_default_are_zero() {
    let initial = StateVersion::INITIAL;
    let explicit = StateVersion::new(0);
    let default = StateVersion::default();

    assert_eq!(initial, explicit);
    assert_eq!(initial, default);
    assert_eq!(initial.as_u64(), 0);
    assert_eq!(u64::from(initial), 0);
    assert_eq!(initial.to_string(), "0");
}

#[test]
fn state_version_checked_next_increments_normal_values() {
    let version = StateVersion::new(41);

    assert_eq!(version.checked_next(), Some(StateVersion::new(42)));
}

#[test]
fn state_version_checked_next_reaches_maximum_from_previous_value() {
    let before_max = StateVersion::new(u64::MAX - 1);

    assert_eq!(before_max.checked_next(), Some(StateVersion::new(u64::MAX)));
}

#[test]
fn state_version_checked_next_returns_none_at_maximum() {
    let max = StateVersion::new(u64::MAX);

    assert_eq!(max.checked_next(), None);
}

#[test]
fn state_version_saturating_next_increments_normal_values() {
    let version = StateVersion::new(41);

    assert_eq!(version.saturating_next(), StateVersion::new(42));
}

#[test]
fn state_version_saturating_next_reaches_maximum_from_previous_value() {
    let before_max = StateVersion::new(u64::MAX - 1);

    assert_eq!(before_max.saturating_next(), StateVersion::new(u64::MAX));
}

#[test]
fn state_version_saturating_next_stays_at_maximum() {
    let max = StateVersion::new(u64::MAX);

    assert_eq!(max.saturating_next(), max);
}

#[test]
fn state_version_is_copy_comparable_ordered_and_hashable() {
    let zero = StateVersion::new(0);
    let duplicate_zero = StateVersion::new(0);
    let one = StateVersion::new(1);

    let copied = zero;

    assert_eq!(zero, copied);
    assert_eq!(zero, duplicate_zero);
    assert_ne!(zero, one);
    assert!(zero < one);

    let mut versions = HashSet::new();
    versions.insert(zero);
    versions.insert(duplicate_zero);
    versions.insert(one);

    assert_eq!(versions.len(), 2);
    assert!(versions.contains(&StateVersion::new(0)));
    assert!(versions.contains(&StateVersion::new(1)));
}

#[test]
fn zobrist_hash_zero_and_default_are_zero() {
    let zero = ZobristHash::ZERO;
    let default = ZobristHash::default();

    assert_eq!(zero, default);
    assert_eq!(zero.as_u64(), 0);
    assert_eq!(u64::from(zero), 0);
}

#[test]
fn zobrist_hash_wraps_raw_value() {
    let hash = ZobristHash::new(42);

    assert_eq!(hash.as_u64(), 42);
    assert_eq!(u64::from(hash), 42);
}

#[test]
fn zobrist_hash_displays_as_fixed_width_lowercase_hex() {
    assert_eq!(ZobristHash::ZERO.to_string(), "0000000000000000");
    assert_eq!(ZobristHash::new(42).to_string(), "000000000000002a");
    assert_eq!(ZobristHash::new(u64::MAX).to_string(), "ffffffffffffffff");
}

#[test]
fn zobrist_hash_is_copy_comparable_ordered_and_hashable() {
    let zero = ZobristHash::new(0);
    let duplicate_zero = ZobristHash::new(0);
    let one = ZobristHash::new(1);

    let copied = zero;

    assert_eq!(zero, copied);
    assert_eq!(zero, duplicate_zero);
    assert_ne!(zero, one);
    assert!(zero < one);

    let mut hashes = HashSet::new();
    hashes.insert(zero);
    hashes.insert(duplicate_zero);
    hashes.insert(one);

    assert_eq!(hashes.len(), 2);
    assert!(hashes.contains(&ZobristHash::new(0)));
    assert!(hashes.contains(&ZobristHash::new(1)));
}

#[test]
fn small_id_error_displays_range_message() {
    let error = SmallIdError::OutOfRange {
        value: 21,
        upper_exclusive: PIECE_COUNT,
    };

    assert_eq!(
        error.to_string(),
        "value 21 is out of range; expected value < 21"
    );
}

#[test]
fn small_id_error_is_copy_comparable_and_hashable() {
    let error = SmallIdError::OutOfRange {
        value: 21,
        upper_exclusive: PIECE_COUNT,
    };

    let copied = error;

    assert_eq!(error, copied);

    let mut errors = HashSet::new();
    errors.insert(error);
    errors.insert(copied);

    assert_eq!(errors.len(), 1);
}

const CONST_PIECE_ZERO: PieceId = match PieceId::try_new(0) {
    Ok(piece) => piece,
    Err(_) => panic!("piece id 0 should be valid in const context"),
};

const CONST_PIECE_ZERO_RAW: u8 = CONST_PIECE_ZERO.as_u8();
const CONST_PIECE_ZERO_INVENTORY_BIT: u32 = CONST_PIECE_ZERO.inventory_bit();

const CONST_ORIENTATION_ZERO: OrientationId = match OrientationId::try_new(0) {
    Ok(orientation) => orientation,
    Err(_) => panic!("orientation id 0 should be valid in const context"),
};

const CONST_ORIENTATION_ZERO_RAW: u8 = CONST_ORIENTATION_ZERO.as_u8();

const CONST_INITIAL_VERSION: StateVersion = StateVersion::INITIAL;
const CONST_NEXT_VERSION: Option<StateVersion> = CONST_INITIAL_VERSION.checked_next();
const CONST_SATURATING_VERSION: StateVersion = CONST_INITIAL_VERSION.saturating_next();

const CONST_ZERO_HASH: ZobristHash = ZobristHash::ZERO;
const CONST_HASH_VALUE: ZobristHash = ZobristHash::new(42);

#[test]
fn const_context_apis_work_for_compact_ids() {
    assert_eq!(CONST_PIECE_ZERO_RAW, 0);
    assert_eq!(CONST_PIECE_ZERO_INVENTORY_BIT, 1);

    assert_eq!(CONST_ORIENTATION_ZERO_RAW, 0);
}

#[test]
fn const_context_apis_work_for_state_version() {
    assert_eq!(CONST_INITIAL_VERSION, StateVersion::new(0));
    assert_eq!(CONST_NEXT_VERSION, Some(StateVersion::new(1)));
    assert_eq!(CONST_SATURATING_VERSION, StateVersion::new(1));
}

#[test]
fn const_context_apis_work_for_zobrist_hash() {
    assert_eq!(CONST_ZERO_HASH, ZobristHash::new(0));
    assert_eq!(CONST_HASH_VALUE.as_u64(), 42);
}
