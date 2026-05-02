//! Transposition table primitives for search implementations.

use crate::{LegalMove, ZobristHash};

const AGE_REPLACE_DISTANCE: u8 = 16;

/// Alpha-beta bound type stored in a transposition entry.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum TranspositionBound {
    /// Fully searched exact score.
    Exact,
    /// Score is a lower bound.
    LowerBound,
    /// Score is an upper bound.
    UpperBound,
}

/// Cached search result for one position hash.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct TranspositionEntry {
    /// Full position hash used for collision verification.
    pub hash: ZobristHash,
    /// Search depth used when this entry was stored.
    pub depth: u8,
    /// Search score.
    pub score: i16,
    /// Best move found by the caller's search, if available.
    pub best_move: Option<LegalMove>,
    /// Alpha-beta bound type.
    pub bound: TranspositionBound,
    /// Caller-managed generation counter for replacement decisions.
    pub age: u8,
}

impl TranspositionEntry {
    /// Creates a transposition table entry.
    #[must_use]
    pub const fn new(
        hash: ZobristHash,
        depth: u8,
        score: i16,
        best_move: Option<LegalMove>,
        bound: TranspositionBound,
        age: u8,
    ) -> Self {
        Self {
            hash,
            depth,
            score,
            best_move,
            bound,
            age,
        }
    }
}

/// Fixed-size direct-mapped transposition table.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct TranspositionTable {
    entries: Vec<Option<TranspositionEntry>>,
    mask: usize,
    len: usize,
}

impl TranspositionTable {
    /// Creates a table with capacity rounded up to a power of two.
    #[must_use]
    pub fn new(entry_count: usize) -> Self {
        let capacity = entry_count.max(1).next_power_of_two();

        Self {
            entries: vec![None; capacity],
            mask: capacity - 1,
            len: 0,
        }
    }

    /// Returns the table capacity.
    #[must_use]
    pub fn capacity(&self) -> usize {
        self.entries.len()
    }

    /// Returns the number of occupied slots.
    #[must_use]
    pub const fn len(&self) -> usize {
        self.len
    }

    /// Returns whether the table has no occupied slots.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Removes all entries while retaining capacity.
    pub fn clear(&mut self) {
        self.entries.fill(None);
        self.len = 0;
    }

    /// Probes the table for an exact hash match.
    #[must_use]
    pub fn probe(&self, hash: ZobristHash) -> Option<&TranspositionEntry> {
        let entry = self.entries[self.index(hash)].as_ref()?;

        if entry.hash == hash {
            Some(entry)
        } else {
            None
        }
    }

    /// Stores an entry if the target slot is empty or replacement is justified.
    pub fn store(&mut self, entry: TranspositionEntry) {
        let index = self.index(entry.hash);

        match self.entries[index] {
            None => {
                self.entries[index] = Some(entry);
                self.len += 1;
            }
            Some(existing) if should_replace(existing, entry) => {
                self.entries[index] = Some(entry);
            }
            Some(_) => {}
        }
    }

    fn index(&self, hash: ZobristHash) -> usize {
        let mask = u64::try_from(self.mask)
            .unwrap_or_else(|_| unreachable!("table mask always fits in u64"));

        usize::try_from(hash.as_u64() & mask)
            .unwrap_or_else(|_| unreachable!("masked hash always fits in usize"))
    }
}

fn should_replace(existing: TranspositionEntry, incoming: TranspositionEntry) -> bool {
    incoming.depth >= existing.depth
        || (incoming.bound == TranspositionBound::Exact
            && existing.bound != TranspositionBound::Exact)
        || incoming.age.wrapping_sub(existing.age) >= AGE_REPLACE_DISTANCE
}
