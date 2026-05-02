use blocus_core::{BlocusEngine, PieceId, standard_repository};

fn piece_id(value: u8) -> PieceId {
    let Ok(piece_id) = PieceId::try_new(value) else {
        panic!("piece id should be valid");
    };

    piece_id
}

#[test]
fn standard_repository_is_initialized_once_and_reused() {
    let first = standard_repository();
    let second = standard_repository();

    assert!(std::ptr::eq(first, second));
    assert_eq!(first.pieces().len(), 21);
    assert_eq!(first.piece(piece_id(0)).name(), "I1");
    assert_eq!(first.piece(piece_id(20)).name(), "Z5");
}

#[test]
fn repeated_engines_share_the_same_piece_repository() {
    let first = BlocusEngine::new();
    let second = BlocusEngine::new();

    assert!(std::ptr::eq(
        first.piece_repository(),
        second.piece_repository()
    ));
    assert!(std::ptr::eq(
        first.piece_repository(),
        standard_repository()
    ));
}

#[test]
fn engine_with_repository_remains_send_sync_copy_and_default() {
    fn assert_send_sync<T: Send + Sync>() {}
    fn assert_copy<T: Copy>() {}

    assert_send_sync::<BlocusEngine>();
    assert_copy::<BlocusEngine>();

    let engine = BlocusEngine::default();
    let copied = engine;

    assert_eq!(engine, copied);
    assert!(std::ptr::eq(
        engine.piece_repository(),
        standard_repository()
    ));
}
