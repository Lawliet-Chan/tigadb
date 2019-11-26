use core::arch::x86_64::{_mm_cmpeq_epi8, _mm_loadu_si128, _mm_movemask_epi8, _mm_set1_epi8};
use std::u32;

const NODE4MIN: usize = 2;
const NODE4MAX: usize = 4;

const NODE16MIN: usize = 5;
const NODE16MAX: usize = 16;

const NODE48MIN: usize = 17;
const NODE48MAX: usize = 48;

const NODE256MIN: usize = 49;
const NODE256MAX: usize = 256;

const PREFIX_LEN: usize = 10;

#[derive(PartialEq)]
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
}

struct Node {
    typ: ArtNodeType,
    keys: Vec<u8>,
    children: Vec<Node>,
    leaf: Option<Leaf>,
}

struct Leaf {}

impl Node {
    #[inline]
    fn new_node4(keys: Vec<u8>) -> Node {
        // keys: [u8;4]
        Node {
            typ: ArtNodeType::Node4,
            keys,
            children: Vec::with_capacity(NODE4MAX),
            leaf: None,
        }
    }

    #[inline]
    fn new_node16(keys: Vec<u8>) -> Node {
        // keys: [u8;16]
        Node {
            typ: ArtNodeType::Node16,
            keys,
            children: Vec::with_capacity(NODE16MAX),
            leaf: None,
        }
    }

    #[inline]
    fn new_node48(keys: Vec<u8>) -> Node {
        // keys: [u8;256]
        Node {
            typ: ArtNodeType::Node48,
            keys,
            children: Vec::with_capacity(NODE48MAX),
            leaf: None,
        }
    }

    #[inline]
    fn new_node256() -> Node {
        Node {
            typ: ArtNodeType::Node256,
            keys: vec![],
            children: Vec::with_capacity(NODE256MAX),
            leaf: None,
        }
    }

    #[inline]
    fn new_leaf_node() -> Leaf {
        Leaf {}
    }

    #[inline]
    fn find_child(&self, k: u8) -> Option<&Node> {
        match &self.typ {
            ArtNodeType::Node4 => {
                for i in 0..self.get_child_size() {
                    if *self.keys.get(i).unwrap() == k {
                        return self.children.get(i);
                    }
                }
                None
            }
            ArtNodeType::Node16 => unsafe {
                let key = _mm_set1_epi8(k as i8);
                let key2 = _mm_loadu_si128(self.keys.as_ptr() as *const _);
                let cmp = _mm_cmpeq_epi8(key, key2);
                let mask = (1 << self.get_child_size()) - 1;
                let bit_field = _mm_movemask_epi8(cmp) & (mask as i32);
                if bit_field > 0 {
                    let u32_bit_field = bit_field as u32;
                    self.children.get(u32_bit_field.trailing_zeros())
                } else {
                    None
                }
            },
            ArtNodeType::Node48 => self.children.get(self.keys.get(k)),
            ArtNodeType::Node256 => self.children.get(k),
        }
    }

    #[inline]
    fn add_child(&mut self, key: u8, node: Node) {
        if self.is_full() {
            self.grow();
            self.add_child(key, node);
            return;
        }
        let size = self.get_child_size();
        match &self.typ {
            ArtNodeType::Node4 => {
                let mut idx = 0;
                for idx in 0..size {
                    if key < self.keys.get(idx) {
                        break;
                    }
                }

                for i in size..idx {
                    if *self.keys.get(i - 1).unwrap() > key {
                        if let Some(k) = self.keys.get_mut(i) {
                            *k = *self.keys.get(i - 1).unwrap();
                        }
                        if let Some(ch) = self.children.get_mut(i) {
                            *ch = *self.children.get(i - 1).unwrap();
                        }
                    }
                }

                if let Some(k) = self.keys.get_mut(idx) {
                    *k = key;
                }

                if let Some(ch) = self.children.get_mut(idx) {
                    *ch = node;
                }
            }
            ArtNodeType::Node16 => {
                self.children.push(node);
                self.keys.push(key);
            }
            ArtNodeType::Node48 => {
                if let Some(k) = self.keys.get_mut(key) {
                    *k = size;
                }
                if let Some(ch) = self.children.get_mut(size - 1) {
                    *ch = node;
                }
            }
            ArtNodeType::Node256 => {
                if let Some(ch) = self.children.get_mut(key) {
                    *ch = node;
                }
            }
        }
    }

    #[inline]
    fn delete_child(&mut self) {
        match &self.typ {
            ArtNodeType::Node4 => {}
            ArtNodeType::Node16 => {}
            ArtNodeType::Node48 => {}
            ArtNodeType::Node256 => {}
        }

        if self.is_less() {
            self.shrink();
        }
    }

    #[inline]
    fn grow(&mut self) {
        match &self.typ {
            ArtNodeType::Node4 => {
                let new_node = Node::new_node16(self.keys.clone());

                *self = new_node;
            }
            ArtNodeType::Node16 => {
                let new_node = Node::new_node48(self.keys.clone());

                *self = new_node;
            }
            ArtNodeType::Node48 => {
                let new_node = Node::new_node256();

                *self = new_node;
            }
            _ => {}
        }
    }

    #[inline]
    fn shrink(&mut self) {
        match &self.typ {
            ArtNodeType::Node16 => {
                let new_node = Node::new_node4(self.keys.clone());

                *self = new_node;
            }
            ArtNodeType::Node48 => {
                let new_node = Node::new_node16(self.keys.clone());

                *self = new_node;
            }
            ArtNodeType::Node256 => {
                let new_node = Node::new_node48(self.keys.clone());

                *self = new_node;
            }
            _ => {}
        }
    }

    #[inline]
    fn is_full(&self) -> bool {
        self.get_child_size() == self.max_size()
    }

    #[inline]
    fn is_less(&self) -> bool {
        self.get_child_size() < self.min_size()
    }

    #[inline]
    fn is_leaf(&self) -> bool {
        self.leaf.is_some()
    }

    #[inline]
    fn get_child_size(&self) -> usize {
        self.children.len()
    }

    #[inline]
    fn max_size(&self) -> usize {
        match &self.typ {
            ArtNodeType::Node4 => NODE4MAX,
            ArtNodeType::Node16 => NODE16MAX,
            ArtNodeType::Node48 => NODE48MAX,
            ArtNodeType::Node256 => NODE256MAX,
        }
    }

    #[inline]
    fn min_size(&self) -> usize {
        match &self.typ {
            ArtNodeType::Node4 => NODE4MIN,
            ArtNodeType::Node16 => NODE16MIN,
            ArtNodeType::Node48 => NODE48MIN,
            ArtNodeType::Node256 => NODE256MIN,
        }
    }
}
