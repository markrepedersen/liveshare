use {
    rand::{thread_rng, Rng},
    std::net::{TcpListener, TcpStream},
};

pub type SiteId = u64;
pub type Clock = u64;
pub type PositionId = u64;

#[derive(Debug, Clone)]
pub struct Id(PositionId, SiteId);

#[derive(Debug, Clone)]
pub struct Position(Vec<Id>);

#[derive(Debug, Clone)]
pub struct AtomId(Position, Clock);

#[derive(Debug, Clone)]
pub struct DocumentAtom(AtomId, String);

#[derive(Debug, Clone)]
pub struct Document {
    site_id: u64,
    sequence: Vec<DocumentAtom>,
}

impl<'a> Document {
    pub fn new() -> Self {
        Self {
            site_id: 0,
            sequence: vec![
                DocumentAtom(AtomId(Position(vec![Id(u64::MIN, 0)]), 0), String::from("")),
                DocumentAtom(AtomId(Position(vec![Id(u64::MAX, 0)]), 0), String::from("")),
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

    fn new_id(low: u64, high: u64) -> u64 {
        let mut rand = thread_rng();
        rand.gen_range(low, high)
    }

    fn binary_search(&self) {}
}
