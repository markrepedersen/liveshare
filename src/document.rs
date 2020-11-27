use rand::thread_rng;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::cmp::{max, min};

const NIL: char = '\0';
const PAGE_MIN: u64 = 0;
const PAGE_MAX: u64 = u64::MAX;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Id {
    digit: u64,
    site: u64,
}

impl Id {
    pub fn new(digit: u64, site_id: u64) -> Self {
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

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Position(Vec<Id>);

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
    pub fn create(site: u64, before: &[Id], after: &[Id]) -> Self {
        let (virtual_min, virtual_max) = (Id::new(PAGE_MIN, site), Id::new(PAGE_MAX, site));
        let len = max(before.len(), after.len());
        let mut new_pos = Vec::new();
        let mut is_same_site = true;

        for i in 0..len {
            let id1 = before.get(i).unwrap_or(&virtual_min);
            let id2 = after
                .get(i)
                .filter(|_| is_same_site)
                .unwrap_or(&virtual_max);
            let diff = id2.digit - id1.digit;

            if diff > 1 {
                let new_digit = Self::generate_random_digit(id1.digit, id2.digit);
                new_pos.push(Id::new(new_digit, site));
                break;
            } else {
                new_pos.push(id1.to_owned());
                is_same_site = id1.cmp(id2) == Ordering::Equal;
            }
        }

        Position(new_pos)
    }

    fn generate_random_digit(lower_bound: u64, upper_bound: u64) -> u64 {
        let mut rand = thread_rng();
        rand.gen_range(lower_bound, upper_bound)
    }
}

/// This is the smallest unit of change in a document.
/// When users change an individual character in a document, this struct will be used to denote it.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct Char {
    clock: u64,
    val: char,
    position: Position,
}

impl Ord for Char {
    fn cmp(&self, other: &Self) -> Ordering {
        self.position.cmp(&other.position)
    }
}

impl PartialOrd for Char {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Char {
    fn new(position: Position, clock: u64, val: char) -> Self {
        Self {
            position,
            clock,
            val,
        }
    }

    pub fn create(c: char, site: u64, c1: &Char, c2: &Char) -> Self {
        Self {
            position: Position::create(site, &c1.position.0, &c2.position.0),
            clock: 0,
            val: c,
        }
    }
}

#[derive(Debug)]
pub struct Document {
    pub nodes: Vec<Char>,
    pub site: u64,
}

impl Document {
    pub fn new() -> Self {
        Self {
            nodes: vec![
                Char::new(Position(vec![Id::new(PAGE_MIN, 0)]), 0, NIL),
                Char::new(Position(vec![Id::new(PAGE_MAX, 0)]), 0, NIL),
            ],
            site: 0,
        }
    }

    /// On receiving an remote INSERT operation with position identifier p, a client performs the following:
    /// A client makes a local change at index i and the following steps are performed:
    /// - Finds the i-th and (i+1)th position identifier.
    /// - Inserts a new position identifier between them.
    /// - Sends a remote INSERTION operation (with the newly generated position identifier) to all other clients.
    pub fn insert_by_index(&mut self, c: char, n: usize) -> Option<&Char> {
        let prev = self.nodes.get(n)?;
        let next = self.nodes.get(n + 1)?;
        let node = Char::create(c, self.site, prev, next);

        self.nodes.insert(n + 1, node);
        self.nodes.get(n + 1)
    }

    /// Receives a local request to delete an atom from the document.
    /// A client deletes a character at index i and the following steps are performed:
    /// - Find the (i+1)-th character in the document (i + 1 since we don't count the virtual nodes).
    /// - Record its position identifer and then deletes it from the document.
    /// - Sends a remote DELETE operation (with the newly generated position identfier) to all other clients.
    pub fn delete_by_index(&mut self, n: usize) -> Option<Char> {
        let node = self.nodes.remove(n + 1);
        Some(node)
    }

    /// Gets the content of the document by aggregating all of the nodes together into a single string.
    pub fn content(&self) -> Option<String> {
        Some(self.nodes.iter().fold(String::new(), |mut acc, c| {
            if c.val != NIL {
                acc.push(c.val);
            }
            acc
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::Document;
    use super::PAGE_MAX;
    use super::PAGE_MIN;

    fn is_sorted(doc: &Document) -> bool {
        doc.nodes.windows(2).all(|window| window[0] < window[1])
    }

    #[test]
    fn test_initial_insert() {
        let mut doc = Document::new();
        doc.insert_by_index('a', 0);
        assert_eq!(doc.nodes.len(), 3);

        let digit = doc.nodes[1].position.0[0].digit;

        assert!(PAGE_MIN < digit && digit < PAGE_MAX);
    }

    #[test]
    fn test_consecutive_inserts() {
        let mut doc = Document::new();

        doc.insert_by_index('h', 0);
        doc.insert_by_index('e', 1);
        doc.insert_by_index('l', 2);
        doc.insert_by_index('l', 3);
        doc.insert_by_index('o', 4);
        doc.insert_by_index(' ', 5);
        doc.insert_by_index('w', 6);
        doc.insert_by_index('o', 7);
        doc.insert_by_index('r', 8);
        doc.insert_by_index('l', 9);
        doc.insert_by_index('d', 10);

        assert_eq!(doc.content().unwrap(), "hello world");
        assert_eq!(
            doc.nodes.windows(2).all(|window| window[0] < window[1]),
            true
        );
    }

    #[test]
    fn test_interleaved_inserts() {
        let mut doc = Document::new();

        doc.insert_by_index('h', 0);
        doc.insert_by_index('e', 0);
        doc.insert_by_index('l', 0);
        doc.insert_by_index('l', 0);
        doc.insert_by_index('o', 0);
        doc.insert_by_index(' ', 0);
        doc.insert_by_index('w', 0);
        doc.insert_by_index('o', 0);
        doc.insert_by_index('r', 0);
        doc.insert_by_index('l', 0);
        doc.insert_by_index('d', 0);

        assert_eq!(doc.content().unwrap(), "dlrow olleh");
        assert_eq!(is_sorted(&doc), true);
    }

    #[test]
    fn test_delete() {
        let mut doc = Document::new();

        doc.insert_by_index('h', 0);
        doc.insert_by_index('e', 0);
        doc.insert_by_index('l', 0);
        doc.insert_by_index('l', 0);
        doc.insert_by_index('o', 0);
        doc.insert_by_index(' ', 0);
        doc.insert_by_index('w', 0);
        doc.insert_by_index('o', 0);
        doc.insert_by_index('r', 0);
        doc.insert_by_index('l', 0);
        doc.insert_by_index('d', 0);
        doc.delete_by_index(5);

        assert_eq!(doc.content().unwrap(), "dlrowolleh");
        assert_eq!(is_sorted(&doc), true);
    }
}
