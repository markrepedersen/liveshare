use crate::{
    document::{PAGE_MAX, PAGE_MIN},
    id::Id,
};
use rand::{thread_rng, Rng};
use serde::{Deserialize, Serialize};
use std::cmp::{max, min, Ordering};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Position(pub Vec<Id>);

impl Ord for Position {
    fn cmp(&self, other: &Self) -> Ordering {
        let (len1, len2) = (self.0.len(), other.0.len());

        for i in 0..min(len1, len2) {
            let ord = self.0[i].cmp(&other.0[i]);
            if ord != Ordering::Equal {
                return ord;
            }
        }

        if len1 < len2 {
            return Ordering::Less;
        } else if len1 > len2 {
            return Ordering::Greater;
        } else {
            return Ordering::Equal;
        }
    }
}

impl PartialOrd for Position {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<Idx> std::ops::Index<Idx> for Position
where
    Idx: std::slice::SliceIndex<[Id]>,
{
    type Output = Idx::Output;

    fn index(&self, index: Idx) -> &Self::Output {
        &self.0[index]
    }
}

impl Position {
    pub fn new(ids: &[Id]) -> Self {
        Self(ids.to_vec())
    }

    /// Creates a new position identifier based on the following cases.
    /// # Case 1: Digits differ by exactly 1
    /// In this case, we can't find an integer that lies between the two digits.
    /// Therefore, we must continue onto the next `Id`.
    /// ```
    ///   prev  (0.1311) : [1,1] -> *[3,1]* -> [1,1] -> [1,1] -> ..
    ///   next  (0.1411) : [1,1] -> *[4,1]* -> [1,1] -> [1,1] -> ..
    /// ```
    /// # Case 2: Digits differ by more than 1
    /// We can create a new identifier between the two digits
    /// Note that the length of `between` will not be larger than `prev` or `next` in this case
    /// ```
    ///   prev  (0.1359) : [1,1] -> *[3,1]* -> [5,3] -> [9,2]
    ///   next  (0.1610) : [1,1] -> *[6,1]* -> [10,1]
    /// between (0.1500) : [1,1] ->  [5,1]
    /// ```
    /// # Case 3: Same digits, different site
    /// ```
    ///   prev  (0.13590) : [1,1] -> *[3,1]* -> [5,3] -> [9,2]
    ///   next  (0.13800) : [1,1] -> *[3,3]* -> [8,1]
    /// between (0.13591) : [1,1] ->  [3,1]  -> [5,3] -> [9,2] -> [1,1]
    pub fn create(site: i64, before: &[Id], after: &[Id]) -> Self {
        let (virtual_min, virtual_max) = (Id::new(PAGE_MIN, site), Id::new(PAGE_MAX, site));
        let max_len = max(before.len(), after.len());
        let mut new_pos = Vec::new();
        let mut is_same_site = true;
        let mut did_change = false;

        for i in 0..max_len {
            let id1 = before.get(i).unwrap_or(&virtual_min);
            let id2 = after
                .get(i)
                .filter(|_| is_same_site)
                .unwrap_or(&virtual_max);
            let diff = id2.digit - id1.digit;

            if diff > 1 {
                // Both digits differ by more than 1, so generate a random digit between the two ID digits, exclusively.
                let new_digit = Self::generate_random_digit(id1.digit + 1, id2.digit);
                did_change = true;
                new_pos.push(Id::new(new_digit, site));
                break;
            } else {
                // Both IDs differ by at most 1
                new_pos.push(id1.to_owned());
                is_same_site = id1.cmp(id2) == Ordering::Equal;
            }
        }

        if !did_change {
            // In this case, the digits at each i-th ID differed by at most one and each position had the same length.
            // If this case wasn't here, then each ID will simply be appended each at step, so you'll get the same position as the n-th position, which isn't good.
            let new_digit = Self::generate_random_digit(virtual_min.digit + 1, virtual_max.digit);
            new_pos.push(Id::new(new_digit, site));
        }

        Position(new_pos)
    }

    fn generate_random_digit(lower_bound: u64, upper_bound: u64) -> u64 {
        let mut rand = thread_rng();
        rand.gen_range(lower_bound, upper_bound)
    }
}
