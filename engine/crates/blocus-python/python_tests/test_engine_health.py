import blocus_engine


def test_engine_health_returns_true() -> None:
    assert blocus_engine.engine_health() is True
