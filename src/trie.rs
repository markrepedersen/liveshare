use {
    crate::document::Atom,
    rand::{thread_rng, Rng},
    std::collections::HashMap,
};

#[derive(Default, Debug, Clone)]
struct TrieNode {
    pub children: HashMap<Atom, TrieNode>,
    pub is_end: bool,
}

impl TrieNode {
    #[inline]
    pub fn new() -> Self {
        Self {
            children: HashMap::new(),
            is_end: false,
        }
    }

    fn contains_key(&self, key: &Atom) -> bool {
        self.children.contains_key(&key)
    }

    #[inline]
    pub fn get_child(&self, key: &Atom) -> Option<&TrieNode> {
        self.children().get(&key)
    }

    #[inline]
    pub fn get_child_mut(&mut self, key: &Atom) -> Option<&mut TrieNode> {
        self.children_mut().get_mut(&key)
    }

    #[inline]
    pub fn children(&self) -> &HashMap<Atom, TrieNode> {
        &self.children
    }

    #[inline]
    pub fn children_mut(&mut self) -> &mut HashMap<Atom, TrieNode> {
        &mut self.children
    }

    #[inline]
    pub fn set_end(&mut self) {
        self.is_end = true;
    }
}

/// Trie implementation for string prefix matching.
#[derive(Default, Debug)]
pub struct Trie {
    root: TrieNode,
}

impl Trie {
    #[inline]
    pub fn new() -> Self {
        Self {
            root: TrieNode::new(),
        }
    }

    #[inline]
    fn root(&self) -> &TrieNode {
        &self.root
    }

    #[inline]
    fn root_mut(&mut self) -> &mut TrieNode {
        &mut self.root
    }

    /**
    Generates a new ID between two atoms.
     */
    #[inline]
    pub fn new_id(low: u64, high: u64) -> u64 {
        let mut rand = thread_rng();
        rand.gen_range(low, high)
    }

    /**
    Checks if a sequence of atoms can be found.
    */
    #[inline]
    pub fn contains(&self, atoms: &[Atom]) -> bool {
        let mut node = self.root();
        for atom in atoms {
            if node.contains_key(atom) {
                node = node.get_child(atom).unwrap();
            } else {
                return false;
            }
        }

        node.is_end
    }

    /**
    Inserts an atom by its index.
    When a user inserts a character at a specific index, then this method will be called.
    */
    #[inline]
    pub fn insert(&mut self, word: &Atom) {
        let mut cur_node = self.root_mut();
        for atom in atoms {
            if cur_node.contains_key(atom) {
                cur_node = cur_node.get_child_mut(atom).unwrap();
            } else {
                cur_node.children_mut().insert(c, TrieNode::new());
                cur_node = cur_node.get_child_mut(c).unwrap();
            }
        }

        cur_node.set_end();
    }
}

#[cfg(test)]
mod tests {
    //     use super::*;

    //     #[test]
    //     fn test_insertion() {
    //         let mut trie = Trie::new();
    //         trie.insert("tr");
    //         trie.insert("tri");
    //         trie.insert("trie");
    //         trie.insert("hello");

    //         assert!(trie.contains("tr"));
    //         assert!(trie.contains("tri"));
    //         assert!(trie.contains("trie"));
    //         assert!(trie.contains("hello"));
    //         assert!(!trie.contains("hell"));
    //     }

    //     #[test]
    //     fn test_longest_prefix_sad_path() {
    //         let mut trie = Trie::new();
    //         trie.insert("tr");
    //         trie.insert("tri");
    //         trie.insert("trie");
    //         trie.insert("hello");

    //         assert_eq!(trie.longest_prefix("hello mark"), None);
    //     }

    //     #[test]
    //     fn test_longest_prefix_happy_path() {
    //         let mut trie = Trie::new();
    //         trie.insert("tr");
    //         trie.insert("tri");
    //         trie.insert("trie");
    //         trie.insert("hello");

    //         assert_eq!(trie.longest_prefix("trie"), Some("trie".to_string()));
    //         assert_eq!(trie.longest_prefix("hello"), Some("hello".to_string()));
    //     }
}
