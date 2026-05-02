use blocus_core::engine_health;

#[test]
fn engine_health_returns_true() {
    assert!(engine_health());
}

#[test]
fn engine_health_is_reexported_from_crate_root() {
    let health_check: fn() -> bool = engine_health;
    assert!(health_check());
}
