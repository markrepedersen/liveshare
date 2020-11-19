use {
    std::hash::Hash,
    std::net::{TcpListener, TcpStream},
};

// This type is reserved for the first and last atoms in a document.
pub const NIL: char = '\0';

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Id {
    digit: u64,
    site_id: u64,
}

impl Hash for Id {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.digit.hash(state);
    }
}

/**
An atom by itself is not unique. A sequence of atoms, however, ARE unique.
Sequences of atoms in this implementation are formed by iterating over a path in a Trie.
*/
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Atom {
    id: Id,
    clock: u64,
    val: char,
}

impl Hash for Atom {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

#[derive(Debug, Clone)]
pub struct Document {
    sequence: Vec<Atom>,
}

impl<'a> Document {
    pub fn new() -> Self {
        Self {
            sequence: vec![
                Atom {
                    id: Id {
                        digit: u64::MIN,
                        site_id: 0,
                    },
                    clock: 0,
                    val: NIL,
                },
                Atom {
                    id: Id {
                        digit: u64::MAX,
                        site_id: 0,
                    },
                    clock: 0,
                    val: NIL,
                },
            ],
        }
    }

    /**
    Inserts an atom into the document.

    A client makes a local change at index i and the following steps are performed:
    1. Finds the i-th and (i+1)th position identifier.
    2. Inserts a new position identifier between them.
    3. Sends a remote INSERTION operation (with the newly generated position identfier) to all other clients.

    On receiving an INSERT operation with position identifier p, a client performs the following:
    1. Binary search to find the location to insert p.
    2. Insert p into document.
    */
    pub fn insert(&mut self, i: u64) {}

    /**
    Deletes an atom from the document.

    A client deletes a character at index i and the following steps are performed:
    1. Find the i-th character in the document.
    2. Record its position identifer and then deletes it from the document.
    3. Sends a remote DELETE operation (with the newly generated position identfier) to all other clients.

    On receiving a DELETE operation with position identifier p, a client performs the following:
    1. Binary search to find the location of p.
    2. Deletes p from the document.
    */
    pub fn remove(&mut self, i: u64) {}
}
