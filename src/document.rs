use rand::thread_rng;
use rand::Rng;
use std::cmp::min;
use std::cmp::Ordering;

const NIL: char = '\0';
const PAGE_MIN: u64 = 0;
const PAGE_MAX: u64 = u64::MAX;

/// Converts a number to a vector of its digit parts.
/// Since, for example, 201 == 2010 when comparing positions, trailing zeroes are removed.
const fn number_to_vec(n: u64) -> Vec<u64> {
    let mut digits = Vec::new();
    let mut n = n;

    while n > 9 {
        digits.push(n % 10);
        n = n / 10;
    }

    digits.push(n);
    digits.reverse();

    let mut n = digits.len() - 1;

    while digits[n] == 0 {
        digits.remove(n);
        n -= 1;
    }

    digits
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Id {
    digit: u64,
    site_id: u64,
}

impl Id {
    pub fn new(digit: u64, site_id: u64) -> Self {
        Id { digit, site_id }
    }
}

impl Ord for Id {
    fn cmp(&self, other: &Self) -> Ordering {
        if self.digit < other.digit {
            return Ordering::Less;
        } else if self.digit > other.digit {
            return Ordering::Greater;
        } else {
            if self.site_id < other.site_id {
                return Ordering::Less;
            } else if self.site_id > other.site_id {
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Position(Vec<Id>);

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
    /// When generating a new character between two others, we can do so
    /// by mapping each character's position to a number N s.t. 0 > N > 1.
    /// The delta of these two numbers plus the first number will be the new
    /// position between them, since it is guaranteed that this number will be smaller than the second.
    pub fn create(site: u64, before: &[Id], after: &[Id]) -> Position {
        let n1 = if before.is_empty() {
            PAGE_MIN
        } else {
            before.iter().fold(0, |acc, v| acc + v.digit)
        };
        let n2 = if after.is_empty() {
            PAGE_MAX
        } else {
            after.iter().fold(0, |acc, v| acc + v.digit)
        };
        let mid = number_to_vec(n1 + Position::generate_random_id(n2 - n1));

        Position(
            mid.iter()
                .enumerate()
                .map(|(i, digit)| match *digit {
                    d if i == mid.len() - 1 => Id::new(d, site),
                    d if i < before.len() && d == before[i].digit => Id::new(d, before[i].site_id),
                    d if i < after.len() && d == after[i].digit => Id::new(d, after[i].site_id),
                    d if i < before.len() && d == before[i].digit => Id::new(d, site),
                    d => Id::new(d, site),
                })
                .collect(),
        )
    }

    fn generate_random_id(upper_bound: u64) -> u64 {
        let mut rand = thread_rng();
        rand.gen_range(1, upper_bound)
    }
}

/// This is the smallest unit of change in a document.
/// When users change an individual character in a document, this struct will be used to denote it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Char {
    position: Position,
    clock: u64,
    val: char,
}

impl Ord for Char {
    fn cmp(&self, other: &Self) -> Ordering {
        let (len1, len2) = (self.position.0.len(), other.position.0.len());

        for i in 0..min(len1, len2) {
            let ord = self.position.0[i].cmp(&other.position.0[i]);
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

    /// Creates a new position identifier based on the following rules:
    pub fn create(c: char, site: u64, c1: &Char, c2: &Char) -> Self {
        let (pos1, pos2) = (&c1.position.0, &c2.position.0);
        let res;

        for i in 0..min(pos1.len(), pos2.len()) {
            let (root1, root2) = (pos1[i], pos2[i]);

            if root1.digit != root2.digit {
                res = Char::new(Position::create(site, pos1, pos2), 0, c);
                break;
            } else if root1.site_id < root2.site_id {
                res = Char::new(Position::create(site, &pos1[..i], &[]), 0, c);
                break;
            } else if root1.site_id == root2.site_id {
                continue;
            } else {
                dbg!(c1);
                dbg!(c2);
                panic!("Invalid position ordering.")
            }
        }

        res
    }
}

#[derive(Debug)]
pub struct Document {
    nodes: Vec<Char>,
    site: u64,
}

impl Document {
    /// Inserts an character into the document.
    /// # Local Change
    /// A client makes a local change at index i and the following steps are performed:
    /// - Finds the i-th and (i+1)th position identifier.
    /// - Inserts a new position identifier between them.
    /// - Sends a remote INSERTION operation (with the newly generated position identifier) to all other clients.
    /// # Remote Change
    /// On receiving an INSERT operation with position identifier p, a client performs the following:
    /// - Binary search to find the location to insert p.
    /// - Insert p into document.
    pub fn insert(&mut self, c: char, n: usize) {
        let prev = self.nodes.get(n).expect("Invalid index.");
        let next = self.nodes.get(n + 1).expect("Invalid index.");
        let between = Char::create(c, self.site, prev, next);

        self.nodes.insert(n, between);
    }

    /// Deletes an atom from the document.
    /// # Local Change
    /// A client deletes a character at index i and the following steps are performed:
    /// - Find the i-th character in the document.
    /// - Record its position identifer and then deletes it from the document.
    /// - Sends a remote DELETE operation (with the newly generated position identfier) to all other clients.
    /// # Remote Change
    /// - On receiving a DELETE operation with position identifier p, a client performs the following:
    /// - Binary search to find the location of p.
    /// - Deletes p from the document.
    pub fn remove(&mut self, i: u64) {}
}
