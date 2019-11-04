#![feature(core_intrinsics)]
use std::intrinsics::cttz;

use core::arch::x86_64::{_mm_cmpeq_epi8, _mm_loadu_si128, _mm_movemask_epi8, _mm_set1_epi8};

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
    keys: &'a [u8],
    children: &'a [Node<'a>],
    children_count: usize,
}

impl Node {
    fn new_node4(keys: [u8; 4]) -> Node {
        Node {
            typ: ArtNodeType::Node4,
            keys: &keys,
            children: &[],
            children_count: 0,
        }
    }

    fn new_node16(keys: [u8; 16]) -> Node {
        Node {
            typ: ArtNodeType::Node16,
            keys: &keys,
            children: &[],
            children_count: 0,
        }
    }

    fn new_node48(keys: [u8; 256]) -> Node {
        Node {
            typ: ArtNodeType::Node48,
            keys: &keys,
            children: &[],
            children_count: 0,
        }
    }

    fn new_node256() -> Node {
        Node {
            typ: ArtNodeType::Node256,
            keys: &[],
            children: &[],
            children_count: 0,
        }
    }

    fn new_leaf_node(keys: &[u8]) -> Node {
        Node {
            typ: ArtNodeType::Leaf,
            keys,
            children: &[],
            children_count: 0,
        }
    }

    fn find_child(&self, k: u8) -> Option<Node> {
        match &self.typ {
            ArtNodeType::Node4 => {
                for i in 0..self.children_count {
                    if self.keys[i] == k {
                        Some(self.children[i])
                    }
                }
                None
            }
            ArtNodeType::Node16 => unsafe {
                let key = _mm_set1_epi8(k as i8);
                let key2 = _mm_loadu_si128(self.keys);
                let cmp = _mm_cmpeq_epi8(key, key2);
                let mask = (1 << self.children_count) - 1;
                let bit_field = _mm_movemask_epi8(cmp) & (mask as i32);
                if bit_field {
                    Some(self.children[cttz(bit_field)])
                }
            },
            ArtNodeType::Node48 => Some(self.children[self.keys[k]]),
            ArtNodeType::Node256 => Some(self.children[k]),
            ArtNodeType::Leaf => {}
        }
    }

    fn add_child(&self) {}

    fn delete_child(&self) {}

    fn grow(&self) {}

    fn is_full(&self) -> bool {
        let node_size = self.get_size();
        match &self.typ {
            ArtNodeType::Node4 => node_size == NODE4MAX,
            ArtNodeType::Node16 => node_size == NODE16MAX,
            ArtNodeType::Node48 => node_size == NODE48MAX,
            ArtNodeType::Node256 => node_size == NODE256MAX,
            ArtNodeType::Leaf => true,
        }
    }

    fn is_leaf(&self) -> bool {
        self.typ == ArtNodeType::Leaf
    }

    fn get_size(&self) -> usize {
        0
    }
}
