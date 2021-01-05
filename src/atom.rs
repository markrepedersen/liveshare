use crate::{position::Position, id::Id};
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;

/// This is the smallest unit of change in a document.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct Atom {
    pub clock: u64,
    pub val: char,
    pub position: Position,
}

impl Ord for Atom {
    fn cmp(&self, other: &Self) -> Ordering {
        self.position.cmp(&other.position)
    }
}

impl PartialOrd for Atom {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Atom {
    pub fn new(position: Position, clock: u64, val: char) -> Self {
        Self {
            position,
            clock,
            val,
        }
    }

    pub fn create(c: char, site: i64, c1: &Atom, c2: &Atom) -> Self {
        Self {
            position: Position::create(site, &c1.position.0, &c2.position.0),
            clock: 0,
            val: c,
        }
    }
}
