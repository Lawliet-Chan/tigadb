const NODE4MIN: usize = 2;
const NODE4MAX: usize = 4;

const NODE16MIN: usize = 5;
const NODE16MAX: usize = 16;

const NODE48MIN: usize = 17;
const NODE48MAX: usize = 48;

const NODE256MIN: usize = 49;
const NODE256MAX: usize = 256;

const PREFIX_LEN: usize = 10;

enum ArtNodeType {
    // key 4
    // children 4
    Node4,

    // key 16
    // children 16
    Node16,

    // key 256
    // children 48
    Node48,

    // children 256
    Node256,

    Leaf,
}

struct Node<'a> {
    typ: ArtNodeType,
    key_char: &'a [char],
    children: &'a [Node],
}

impl Node {
    fn new_node4(key_char: [char; 4]) -> Node {
        Node {
            typ: ArtNodeType::Node4,
            key_char: &key_char,
            children: &[],
        }
    }

    fn new_node16(key_char: [char; 16]) -> Node {
        Node {
            typ: ArtNodeType::Node16,
            key_char: &key_char,
            children: &[],
        }
    }

    fn new_node48(key_char: [char; 256]) -> Node {
        Node {
            typ: ArtNodeType::Node48,
            key_char: &key_char,
            children: &[],
        }
    }

    fn new_node256() -> Node {
        Node {
            typ: ArtNodeType::Node256,
            key_char: &[],
            children: &[],
        }
    }

    fn new_leaf_node(key_char: &[char]) -> Node {
        Node {
            typ: ArtNodeType::Leaf,
            key_char,
            children: &[],
        }
    }

    fn is_leaf(&self) -> bool {
        self.typ == ArtNodeType::Leaf
    }

    fn add_child(&self) {}

    fn remove_child(&self) {}

    fn find_child(&self, key: &[char]) {}

    fn grow(&self) {}

    fn is_full(&self) -> bool {}
}
