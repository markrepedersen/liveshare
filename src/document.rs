use crate::{atom::Atom, id::Id, position::Position, range::Point, range::Range};
use std::collections::hash_map::{Entry, HashMap};

pub const NIL: char = '\0';
pub const PAGE_MIN: u64 = 0;
pub const PAGE_MAX: u64 = u64::MAX;

pub struct RemoteInsert {}

pub struct LocalInsert {}

#[derive(Debug)]
pub struct Document {
    nodes: HashMap<usize, Vec<Atom>>,
    site: i64,
}

impl Document {
    /// Creates a new empty document with a given site ID.
    /// Note that the site ID must be unique across all replicated documents.
    pub fn new(site: i64) -> Self {
        Self {
            nodes: HashMap::new(),
            site,
        }
    }

    /// Inserts all characters in `lines` from `start` until `end`.
    /// # Note
    /// Since only the start and end are specified, we'll have to start inserting from the end until the beginning to cover some edge cases.
    /// i.e. start at the end position and keep iterating until the beginning of the line is encountered. Once it is, then
    /// go to the previous line and keep iterating until the start position.
    pub fn local_insert(&mut self, range: &Range, lines: &[char]) {}

    /// Deletes all atoms from `start` until `end`.
    /// # Note
    /// Entire lines may be deleted, changing subsequent row numbers.
    pub fn local_delete(&mut self, range: &Range) {
        let mut curr = range.start;
        let updated_row = None;

        while let Some(mut nodes) = self.line(curr.row) {
            match curr.row {
                0 => {
                    match range.end.column {
                        0 => nodes.splice(range.start.column..range.end.column, Vec::new()),
                        _ => nodes.splice(range.start.column.., Vec::new()),
                    };
                }
                val if val < range.end.row => {
                    nodes.clear();
                }
                val if val == range.end.row => {
                    nodes.splice(0..range.end.column, Vec::new());
                }
                _ => break,
            }

            if nodes.is_empty() {
                updated_row = Some(curr.row);
            }
        }
    }

    /// Inserts all values in `lines` at the position of its head.
    pub fn remote_insert(&mut self, lines: &[Atom]) -> Option<(Vec<char>, Range)> {
        let head = lines.first()?;

        for (row, nodes) in &self.nodes {
            if let Err(i) = nodes.binary_search(head) {
                nodes.splice(i..i, lines.iter().cloned());

                return Some(Range::new((i, i), (i + lines.len(), i)));
            }
        }

        None
    }

    /// Deletes from the first value matching `start_val` until `end_val`.
    pub fn remote_delete(&mut self, lines: &[Atom]) -> Option<(Vec<char>, Range)> {
        for (row, nodes) in &self.nodes {
            if let Ok(start) = nodes.binary_search(lines.first()?) {
                if let Ok(end) = nodes.binary_search(lines.last()?) {
                    nodes.splice(start..end, Vec::new());
                    return Some(());
                }
            }
        }

        None
    }

    /// A client makes a local coordinate change and the following steps are performed:
    /// - Finds the `col`-th and (`col`+1)-th position identifier.
    /// - Inserts a new position identifier between them.
    fn point_insert(&mut self, c: char, point: Point) {
        let row = point.row;
        let col = point.column;
        let prev = self.node(row, col);
        let next = self.node(row, col + 1);
        let node = Atom::create(c, self.site, &prev, &next);

        self.insert_or_update_row(row, col + 1, node)
    }

    /// Inserts `val` by first searching for its correct position n and then inserting it between the n-th and (n+1)th node.
    fn insert_val(&mut self, atom: &Atom) -> Option<Range> {
        for (row, nodes) in &self.nodes {
            match nodes.binary_search(atom) {
                Ok(_) => panic!("Document atoms should always be unique."),
                Err(i) => {
                    nodes.insert(i, atom.to_owned());
                    return Some(Range::new((i, i), (i, i)));
                }
            }
        }

        None
    }

    /// Deletes `val` by first searching for its correct position and then deleting it.
    /// If `val` does not exist inside `nodes`, then return `None`.
    fn delete_val(&mut self, val: &Atom) -> Option<Range> {
        for (row, nodes) in &self.nodes {
            if let Ok(i) = nodes.binary_search(val) {
                self.nodes.remove(&i);
                return Some(Range::new((i, i), (i, i)));
            }
        }

        None
    }

    /// Receives a local request to delete an atom from the document.
    /// A client deletes a character at index i and the following steps are performed:
    /// - Find the (i+1)-th character in the document (i + 1 since we don't count the virtual nodes).
    /// - Record its position identifer and then deletes it from the document.
    fn point_delete(&mut self, row: usize, col: usize) -> Option<Atom> {
        self.nodes
            .get(&row)
            .and_then(|mut nodes| Some(nodes.remove(col)))
    }

    /// Either inserts into an already existing row or creates a new one and inserts into that one.
    fn insert_or_update_row(&mut self, row: usize, col: usize, val: Atom) {
        match self.nodes.entry(row) {
            Entry::Occupied(entry) => {
                let nodes = entry.get();
                nodes.insert(col, val);
            }
            Entry::Vacant(entry) => {
                let mut nodes = entry.insert(Vec::new());
                nodes.push(val);
            }
        }
    }

    /// Gets the content of the document by aggregating all of the nodes together into a single string.
    /// An empty document will produce an empty string.
    fn content(&self) -> String {
        let mut row = 0;
        let mut res = String::new();

        while let Some(nodes) = self.nodes.get(&row) {
            nodes.iter().fold(res, |mut acc, c| {
                if c.val != NIL {
                    acc.push(c.val);
                }
                acc
            });

            row += 1;
        }

        res
    }

    #[inline]
    fn virtual_min(&self) -> Atom {
        Atom::new(Position::new(&vec![Id::new(PAGE_MIN, self.site)]), 0, NIL)
    }

    #[inline]
    fn virtual_max(&self) -> Atom {
        Atom::new(Position::new(&vec![Id::new(PAGE_MAX, self.site)]), 0, NIL)
    }

    /// Get the line for the corresponding `row`.
    fn line(&self, row: usize) -> Option<&Vec<Atom>> {
        self.nodes.get(&row)
    }

    /// Get a specific node.
    /// If the row does not exist, then the `virtual_min` is returned.
    /// If the row exists, but the column does not, then check if the next row contains a value.
    /// If it does, return that one. Otherwise, return `virtual_max`.
    fn node(&self, row: usize, col: usize) -> Atom {
        match self.line(row) {
            Some(nodes) => match nodes.get(col) {
                Some(node) => node.clone(),
                None => match self.line(row + 1) {
                    Some(nodes) => match nodes.first() {
                        Some(node) => node.clone(),
                        None => self.virtual_max(),
                    },
                    None => self.virtual_max(),
                },
            },
            None => self.virtual_min(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Atom;
    use super::Document;
    use super::Id;
    use super::Point;
    use super::Position;
    use super::PAGE_MAX;
    use super::PAGE_MIN;

    fn is_sorted(doc: &Document) -> bool {
        let mut row = 0;
        let mut flattened_doc = Vec::new();

        while let Some(ref mut nodes) = doc.nodes.get(&row) {
            flattened_doc.append(nodes);
            row += 1;
        }

        flattened_doc.windows(2).all(|window| window[0] < window[1])
    }

    #[test]
    fn test_simple_insert_by_position() {
        let mut doc = Document::new(0);

        doc.point_insert('a', Point::new(0, 0));

        assert_eq!(doc.nodes.len(), 1);

        let digit = doc.line(0).unwrap()[1].position.0[0].digit;

        assert!(PAGE_MIN < digit && digit < PAGE_MAX);
    }

    #[test]
    fn test_consecutive_inserts() {
        let mut doc = Document::new(0);

        doc.point_insert('h', Point::new(0, 0));
        doc.point_insert('e', Point::new(0, 1));
        doc.point_insert('l', Point::new(0, 2));
        doc.point_insert('l', Point::new(0, 3));
        doc.point_insert('o', Point::new(0, 4));
        doc.point_insert(' ', Point::new(0, 5));
        doc.point_insert('w', Point::new(0, 6));
        doc.point_insert('o', Point::new(0, 7));
        doc.point_insert('r', Point::new(0, 8));
        doc.point_insert('l', Point::new(0, 9));
        doc.point_insert('d', Point::new(0, 10));

        assert_eq!(doc.content(), "hello world");
        assert!(is_sorted(&doc));
    }

    #[test]
    fn test_insert_by_value() {
        let mut doc = Document::new(0);

        doc.point_insert('h', Point::new(0, 0));
        doc.point_insert('e', Point::new(0, 1));
        doc.point_insert('l', Point::new(0, 2));
        doc.point_insert('l', Point::new(0, 3));
        doc.point_insert('o', Point::new(0, 4));
        doc.point_insert(' ', Point::new(0, 5));
        doc.point_insert('w', Point::new(0, 6));
        doc.point_insert('o', Point::new(0, 7));
        doc.point_insert('r', Point::new(0, 8));
        doc.point_insert('l', Point::new(0, 9));
        doc.point_insert('d', Point::new(0, 10));

        let space = doc.point_delete(0, 6).unwrap().to_owned();

        doc.insert_val(&space);

        assert_eq!(doc.content(), "hello world");
    }

    #[test]
    fn test_insert_by_index_complex() {
        let mut doc = Document::new(0);
        let mut first_row = doc.nodes.insert(0, Vec::new()).unwrap();
        let mut second_row = doc.nodes.insert(1, Vec::new()).unwrap();

        first_row.push(Atom::new(Position(vec![Id::new(1, 0)]), 0, 'h'));

        first_row.push(Atom::new(
            Position(vec![Id::new(1, 0), Id::new(4, 0)]),
            0,
            'h',
        ));

        first_row.push(Atom::new(
            Position(vec![Id::new(1, 0), Id::new(6, 0), Id::new(3, 1)]),
            0,
            'h',
        ));

        second_row.push(Atom::new(
            Position(vec![Id::new(1, 0), Id::new(7, 0)]),
            0,
            'h',
        ));

        second_row.push(Atom::new(Position(vec![Id::new(1, 1)]), 0, 'h'));

        second_row.push(Atom::new(
            Position(vec![Id::new(1, 1), Id::new(1, 1)]),
            0,
            'h',
        ));

        doc.point_insert('a', Point::new(0, 0));
        doc.point_insert('a', Point::new(0, 1));
        doc.point_insert('b', Point::new(0, 6));
        doc.point_insert('c', Point::new(1, 3));

        assert_eq!(is_sorted(&doc), true);
    }
    #[test]
    fn test_delete_by_value_complex() {
        let mut doc = Document::new(0);

        let mut first_row = doc.nodes.insert(0, Vec::new()).unwrap();

        let deleted_node = Atom::new(
            Position(vec![Id::new(1, 0), Id::new(6, 0), Id::new(3, 1)]),
            0,
            'h',
        );

        first_row.push(Atom::new(Position(vec![Id::new(1, 0)]), 0, 'h'));

        first_row.push(Atom::new(
            Position(vec![Id::new(1, 0), Id::new(4, 0)]),
            0,
            'h',
        ));

        first_row.push(deleted_node.to_owned());

        first_row.push(Atom::new(
            Position(vec![Id::new(1, 0), Id::new(7, 0)]),
            0,
            'h',
        ));

        first_row.push(Atom::new(Position(vec![Id::new(1, 1)]), 0, 'h'));

        first_row.push(Atom::new(
            Position(vec![Id::new(1, 1), Id::new(1, 1)]),
            0,
            'h',
        ));

        doc.delete_val(&deleted_node);

        assert_eq!(first_row.contains(&deleted_node), false);
    }

    #[test]
    fn test_delete_by_value() {
        let mut doc = Document::new(0);

        doc.point_insert('h', Point::new(0, 0));
        doc.point_insert('e', Point::new(0, 1));
        doc.point_insert('l', Point::new(0, 2));
        doc.point_insert('l', Point::new(0, 3));
        doc.point_insert('o', Point::new(0, 4));
        doc.point_insert(' ', Point::new(0, 5));
        doc.point_insert('w', Point::new(0, 6));
        doc.point_insert('o', Point::new(0, 7));
        doc.point_insert('r', Point::new(0, 8));
        doc.point_insert('l', Point::new(0, 9));
        doc.point_insert('d', Point::new(0, 10));

        let space = doc.line(0).unwrap().get(6).unwrap().to_owned();

        doc.delete_val(&space);

        assert_eq!(doc.content(), "helloworld");
    }

    #[test]
    fn test_interleaved_inserts() {
        let mut doc = Document::new(0);

        doc.point_insert('h', Point::new(0, 0));
        doc.point_insert('e', Point::new(0, 0));
        doc.point_insert('l', Point::new(0, 0));
        doc.point_insert('l', Point::new(0, 0));
        doc.point_insert('o', Point::new(0, 0));
        doc.point_insert(' ', Point::new(0, 0));
        doc.point_insert('w', Point::new(0, 0));
        doc.point_insert('o', Point::new(0, 0));
        doc.point_insert('r', Point::new(0, 0));
        doc.point_insert('l', Point::new(0, 0));
        doc.point_insert('d', Point::new(0, 0));

        assert_eq!(doc.content(), "dlrow olleh");
        assert_eq!(is_sorted(&doc), true);
    }

    #[test]
    fn test_delete() {
        let mut doc = Document::new(0);

        doc.point_insert('h', Point::new(0, 0));
        doc.point_insert('e', Point::new(0, 0));
        doc.point_insert('l', Point::new(0, 0));
        doc.point_insert('l', Point::new(0, 0));
        doc.point_insert('o', Point::new(0, 0));
        doc.point_insert(' ', Point::new(0, 0));
        doc.point_insert('w', Point::new(0, 0));
        doc.point_insert('o', Point::new(0, 0));
        doc.point_insert('r', Point::new(0, 0));
        doc.point_insert('l', Point::new(0, 0));
        doc.point_insert('d', Point::new(0, 0));

        let content = doc.content();
        let index_of_space = content
            .find(' ')
            .expect("Content should contain a space character");

        doc.point_delete(0, index_of_space + 1);

        assert_eq!(doc.content(), "dlrowolleh");
        assert_eq!(is_sorted(&doc), true);
    }

    #[test]
    fn test_insert_by_range() {
        let mut doc = Document::new(0);
        let mut first_row = doc.nodes.insert(0, Vec::new()).unwrap();
        let mut second_row = doc.nodes.insert(1, Vec::new()).unwrap();

        first_row.push(Atom::new(Position(vec![Id::new(1, 0)]), 0, 'h'));

        first_row.push(Atom::new(
            Position(vec![Id::new(1, 0), Id::new(4, 0)]),
            0,
            'h',
        ));

        first_row.push(Atom::new(
            Position(vec![Id::new(1, 0), Id::new(6, 0), Id::new(3, 1)]),
            0,
            'h',
        ));

        second_row.push(Atom::new(
            Position(vec![Id::new(1, 0), Id::new(7, 0)]),
            0,
            'h',
        ));

        second_row.push(Atom::new(Position(vec![Id::new(1, 1)]), 0, 'h'));
    }
}
