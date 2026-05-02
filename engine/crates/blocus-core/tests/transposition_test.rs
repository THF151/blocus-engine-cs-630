use blocus_core::{
    BoardIndex, LegalMove, OrientationId, PieceId, TranspositionBound, TranspositionEntry,
    TranspositionTable, ZobristHash,
};

fn index(row: u8, col: u8) -> BoardIndex {
    BoardIndex::from_row_col(row, col)
        .unwrap_or_else(|_| panic!("row {row}, col {col} should be valid"))
}

fn piece_id(value: u8) -> PieceId {
    PieceId::try_new(value).unwrap_or_else(|_| panic!("piece id {value} should be valid"))
}

fn orientation_id(value: u8) -> OrientationId {
    OrientationId::try_new(value)
        .unwrap_or_else(|_| panic!("orientation id {value} should be valid"))
}

fn legal_move() -> LegalMove {
    LegalMove {
        piece_id: piece_id(0),
        orientation_id: orientation_id(0),
        anchor: index(0, 0),
        score_delta: 1,
    }
}

fn entry(hash: u64, depth: u8, bound: TranspositionBound, age: u8) -> TranspositionEntry {
    TranspositionEntry::new(
        ZobristHash::new(hash),
        depth,
        i16::from(depth),
        Some(legal_move()),
        bound,
        age,
    )
}

#[test]
fn transposition_table_rounds_capacity_and_probes_exact_hashes() {
    let mut table = TranspositionTable::new(3);
    let stored = entry(0x1234, 4, TranspositionBound::Exact, 0);

    assert_eq!(table.capacity(), 4);
    assert!(table.is_empty());

    table.store(stored);

    assert_eq!(table.len(), 1);
    assert_eq!(table.probe(stored.hash), Some(&stored));
    assert_eq!(table.probe(ZobristHash::new(0x1235)), None);
}

#[test]
fn transposition_table_keeps_deeper_collision_entry() {
    let mut table = TranspositionTable::new(2);
    let deeper = entry(0, 8, TranspositionBound::LowerBound, 0);
    let shallow_collision = entry(2, 2, TranspositionBound::UpperBound, 0);

    table.store(deeper);
    table.store(shallow_collision);

    assert_eq!(table.probe(deeper.hash), Some(&deeper));
    assert_eq!(table.probe(shallow_collision.hash), None);
    assert_eq!(table.len(), 1);
}

#[test]
fn transposition_table_replaces_with_deeper_or_exact_entries() {
    let mut table = TranspositionTable::new(2);
    let shallow = entry(0, 2, TranspositionBound::LowerBound, 0);
    let deeper_collision = entry(2, 8, TranspositionBound::UpperBound, 0);
    let exact_collision = entry(4, 1, TranspositionBound::Exact, 0);

    table.store(shallow);
    table.store(deeper_collision);
    assert_eq!(table.probe(shallow.hash), None);
    assert_eq!(table.probe(deeper_collision.hash), Some(&deeper_collision));

    table.store(exact_collision);
    assert_eq!(table.probe(deeper_collision.hash), None);
    assert_eq!(table.probe(exact_collision.hash), Some(&exact_collision));
}

#[test]
fn transposition_table_replaces_old_entries_and_clear_resets_len() {
    let mut table = TranspositionTable::new(1);
    let old = entry(0, 8, TranspositionBound::Exact, 0);
    let newer = entry(1, 1, TranspositionBound::UpperBound, 16);

    table.store(old);
    table.store(newer);
    assert_eq!(table.probe(old.hash), None);
    assert_eq!(table.probe(newer.hash), Some(&newer));

    table.clear();
    assert!(table.is_empty());
    assert_eq!(table.probe(newer.hash), None);
}
