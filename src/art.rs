#[cfg(target_arch = "x86_64")]
use core::arch::x86_64::{_mm_cmpeq_epi8, _mm_loadu_si128, _mm_movemask_epi8, _mm_set1_epi8};

#[cfg(target_arch = "x86")]
use core::arch::x86::{_mm_cmpeq_epi8, _mm_loadu_si128, _mm_movemask_epi8, _mm_set1_epi8};

use crate::art::ArtNodeType::{Node16, Node256, Node4, Node48};
use core::slice::SliceIndex;
use std::fs::create_dir_all;
use std::u32;

const NODE4MIN: usize = 2;
const NODE4MAX: usize = 4;
const NODE4KEYS: usize = 4;

const NODE16MIN: usize = 5;
const NODE16MAX: usize = 16;
const NODE16KEYS: usize = 16;

const NODE48MIN: usize = 17;
const NODE48MAX: usize = 48;
const NODE48KEYS: usize = 256;

const NODE256MIN: usize = 49;
const NODE256MAX: usize = 256;
const NODE256KEYS: usize = 0;

const PREFIX_LEN: usize = 10;

#[derive(PartialEq)]
pub(crate) enum ArtNodeType {
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

pub(crate) struct Node {
    typ: ArtNodeType,
    keys: Vec<u8>,
    children: Vec<Node>,
    leaf: Option<Leaf>,
}

pub(crate) struct Leaf {
    key: Vec<u8>,

    // (value_file_index, offset, length)
    value_pos: (u8, u64, u64),
}

impl Node {
    #[inline]
    pub(crate) fn new_node(typ: ArtNodeType) -> Node {
        let (key_cap, children_cap) = match typ {
            ArtNodeType::Node4 => (NODE4KEYS, NODE4MAX),
            ArtNodeType::Node16 => (NODE16KEYS, NODE16MAX),
            ArtNodeType::Node48 => (NODE48KEYS, NODE48MAX),
            ArtNodeType::Node256 => (NODE256KEYS, NODE256MAX),
        };

        Node {
            typ,
            keys: Vec::with_capacity(key_cap),
            children: Vec::with_capacity(children_cap),
            leaf: None,
        }
    }

    #[inline]
    pub(crate) fn new_leaf_node(key: Vec<u8>, value_pos: (u8, u64, u64)) -> Leaf {
        Leaf { key, value_pos }
    }

    #[inline]
    pub(crate) fn insert(&mut self, key: Vec<u8>, value_pos: (u8, u64, u64)) {}

    #[inline]
    pub(crate) fn get(&self, key: Vec<u8>) {}

    #[inline]
    pub(crate) fn update(&mut self, key: Vec<u8>, value_pos: (u8, u64, u64)) {}

    #[inline]
    pub(crate) fn remove(&mut self, key: Vec<u8>) {}

    #[inline]
    fn index(&self, k: u8) -> Option<usize> {
        match &self.typ {
            ArtNodeType::Node4 => {
                for i in 0..self.get_child_size() {
                    if *self.keys.get(i).unwrap() == k {
                        return Some(i);
                    }
                }
                None
            }

            ArtNodeType::Node16 => unsafe {
                #[cfg(all(
                    any(target_arch = "x86_64", target_arch = "x86"),
                    target_feature = "sse2"
                ))]
                {
                    let key = _mm_set1_epi8(k as i8);
                    let key2 = _mm_loadu_si128(self.keys.as_ptr() as *const _);
                    let cmp = _mm_cmpeq_epi8(key, key2);
                    let mask = (1 << self.get_child_size()) - 1;
                    let bit_field = _mm_movemask_epi8(cmp) & (mask as i32);
                    if bit_field > 0 {
                        let u32_bit_field = bit_field as u32;
                        Some(u32_bit_field.trailing_zeros())
                    } else {
                        None
                    }
                }
            },

            ArtNodeType::Node48 => self.keys.get(k),

            ArtNodeType::Node256 => Some(k as usize),
        }
    }

    #[inline]
    fn find_child(&self, k: u8) -> Option<&Node> {
        if let Some(idx) = self.index(k) {
            self.children.get(idx)
        } else {
            None
        }
    }

    #[inline]
    fn find_child_mut(&mut self, k: u8) -> Option<&mut Node> {
        if let Some(idx) = self.index(k) {
            self.children.get_mut(idx)
        } else {
            None
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
                        self.set_key(i, *self.keys.get(i - 1).unwrap());
                        self.set_child(i, *self.children.get(i - 1).unwrap());
                    }
                }

                self.set_key(idx, key);
                self.set_child(idx, node);
            }

            ArtNodeType::Node16 => {
                self.children.push(node);
                self.keys.push(key);
            }

            ArtNodeType::Node48 => {
                // size as u8 is safe
                // because the most is 255. When size is 256, it turns grow().
                self.set_key(key, size as u8);
                self.children.push(node);
            }

            ArtNodeType::Node256 => {
                self.set_child(key, node);
            }
        }
    }

    #[inline]
    fn delete_child(&mut self, key: u8) {
        if let Some(idx) = self.index(key) {
            match &self.typ {
                ArtNodeType::Node4 | ArtNodeType::Node16 => {
                    self.keys.remove(idx);
                    self.children.remove(idx);
                }

                ArtNodeType::Node48 => {
                    if self.children.get(idx).is_some() {
                        self.keys.remove(idx);
                        self.children.remove(idx);
                    }
                }

                ArtNodeType::Node256 => {
                    if self.children.get(idx).is_some() {
                        self.children.remove(idx);
                    }
                }
            }

            if self.is_less() {
                self.shrink();
            }
        }
    }

    // Node4 --> Node16
    // Node16 --> Node48
    // Node48 --> Node256
    #[inline]
    fn grow(&mut self) {
        match &self.typ {
            ArtNodeType::Node4 => {
                self.typ = Node16;
                self.children.reserve_exact(12);
                self.keys.reserve_exact(12);
            }

            ArtNodeType::Node16 => {
                let mut new_node = Node::new_node(Node48);
                let chsize = self.get_child_size();
                for i in 0..chsize {
                    if let Some(child) = self.children.get(i) {
                        let mut idx: usize = 0;
                        for j in 0..NODE48MAX {
                            if new_node.children.get(idx).is_some() {
                                idx += 1;
                            } else {
                                break;
                            }
                        }
                        new_node.set_child(idx, *child);
                        new_node.set_key(self.keys.get(i), (idx + 1) as u8);
                    }
                }
                *self = new_node;
            }

            ArtNodeType::Node48 => {
                let mut new_node = Node::new_node(Node256);
                let ksize = self.get_keys_size();
                for i in 0..ksize {
                    if let Some(child) = self.find_child(i as u8) {
                        new_node.set_child(i, *child);
                    }
                }
                //new_node.keys.clone_from(&self.keys);
                *self = new_node;
            }
            _ => {}
        }
    }

    // Node256 --> Node48
    // Node48 --> Node16
    // Node16 --> Node4
    #[inline]
    fn shrink(&mut self) {
        match &self.typ {
            ArtNodeType::Node16 => {
                self.typ = Node4;
                self.keys.shrink_to(NODE4KEYS);
                self.children.shrink_to(NODE4MAX);
            }

            ArtNodeType::Node48 => {
                self.typ = Node16;
                self.keys.shrink_to(NODE16KEYS);
                self.children.shrink_to(NODE16MAX);
            }

            ArtNodeType::Node256 => {
                self.typ = Node48;
                self.keys.reserve_exact(NODE48KEYS);
                self.children.shrink_to(NODE48MAX);
            }

            _ => {}
        }
    }

    #[inline]
    fn set_key<I>(&mut self, i: I, key: u8)
    where
        I: SliceIndex<Self>,
    {
        if let Some(k) = self.keys.get_mut(i) {
            *k = key;
        }
    }

    #[inline]
    fn set_child<I>(&mut self, i: I, child: Node)
    where
        I: SliceIndex<Self>,
    {
        if let Some(ch) = self.children.get_mut(i) {
            *ch = child;
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
    fn get_keys_size(&self) -> usize {
        self.keys.len()
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
