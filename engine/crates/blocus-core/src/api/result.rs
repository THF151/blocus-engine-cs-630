//! State-transition result DTO.

use crate::{DomainEvent, DomainResponse, GameState};

/// Result of a successful state-changing engine command.
#[must_use]
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct GameResult {
    /// Next game state.
    pub next_state: GameState,
    /// Deterministic domain events in causal order.
    pub events: Vec<DomainEvent>,
    /// Response summary.
    pub response: DomainResponse,
}
