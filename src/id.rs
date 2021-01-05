use serde::{Deserialize, Serialize};
use std::cmp::Ordering;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Id {
    pub digit: u64,
    pub site: i64,
}

impl Id {
    pub fn new(digit: u64, site_id: i64) -> Self {
        Id {
            digit,
            site: site_id,
        }
    }
}

impl Ord for Id {
    fn cmp(&self, other: &Self) -> Ordering {
        if self.digit < other.digit {
            return Ordering::Less;
        } else if self.digit > other.digit {
            return Ordering::Greater;
        } else {
            if self.site < other.site {
                return Ordering::Less;
            } else if self.site > other.site {
                return Ordering::Greater;
            } else {
                return Ordering::Equal;
            }
        }
    }
}

impl PartialOrd for Id {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
