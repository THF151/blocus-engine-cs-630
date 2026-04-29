use blocus_core::engine_health;

#[test]
fn engine_health_returns_true() {
    assert!(engine_health());
}
