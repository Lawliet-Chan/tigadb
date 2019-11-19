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

    Leaf,
}

struct Node {
    typ: ArtNodeType,
    keys: Box<[u8]>,
    children: Box<[Node]>,
    children_count: usize,
}

impl Node {
    #[inline]
    fn new_node4(keys: [u8; 4]) -> Node {
        Node {
            typ: ArtNodeType::Node4,
            keys: Box::from(keys),
            children: Box::default(),
            children_count: 0,
        }
    }

    #[inline]
    fn new_node16(keys: [u8; 16]) -> Node {
        Node {
            typ: ArtNodeType::Node16,
            keys: Box::from(keys),
            children: Box::default(),
            children_count: 0,
        }
    }

    #[inline]
    fn new_node48(keys: [u8; 256]) -> Node {
        Node {
            typ: ArtNodeType::Node48,
            keys: Box::from(keys),
            children: Box::default(),
            children_count: 0,
        }
    }

    #[inline]
    fn new_node256() -> Node {
        Node {
            typ: ArtNodeType::Node256,
            keys: Box::default(),
            children: Box::default(),
            children_count: 0,
        }
    }

    #[inline]
    fn new_leaf_node(keys: Box<[u8]>) -> Node {
        Node {
            typ: ArtNodeType::Leaf,
            keys,
            children: Box::default(),
            children_count: 0,
        }
    }

    #[inline]
    fn find_child(&self, k: u8) -> Option<&Node> {
        match &self.typ {
            ArtNodeType::Node4 => {
                for i in 0..self.children_count {
                    if self.keys[i] == k {
                        return Some(&self.children[i]);
                    }
                }
                None
            }
            ArtNodeType::Node16 => unsafe {
                let key = _mm_set1_epi8(k as i8);
                let key2 = _mm_loadu_si128(self.keys.as_ptr() as *const _);
                let cmp = _mm_cmpeq_epi8(key, key2);
                let mask = (1 << self.children_count) - 1;
                let bit_field = _mm_movemask_epi8(cmp) & (mask as i32);
                if bit_field > 0 {
                    let u32_bit_field = bit_field as u32;
                    Some(&self.children[u32_bit_field.trailing_zeros() as usize])
                } else {
                    None
                }
            },
            ArtNodeType::Node48 => Some(&self.children[self.keys[k as usize] as usize]),
            ArtNodeType::Node256 => Some(&self.children[k as usize]),
            ArtNodeType::Leaf => None,
        }
    }

    #[inline]
    fn add_child(&mut self, key: u8, node: Node) {
        if self.is_full() {
            self.grow();
            self.add_child(key, node);
            return;
        }
        let size = self.get_count();
        match &self.typ {
            ArtNodeType::Node4 => {
                let mut idx = 0;
                for idx in 0..size {
                    if key < self.keys[idx] {
                        break;
                    }
                }

                for i in size..idx {
                    if self.keys[i - 1] > key {
                        self.keys[i] = self.keys[i - 1];
                        self.children[i] = self.children[i - 1];
                    }
                }

                self.keys[idx] = key;
                self.children[idx] = node;
                self.incr_count();
            }
            ArtNodeType::Node16 => {}
            ArtNodeType::Node48 => {}
            ArtNodeType::Node256 => {}
            ArtNodeType::Leaf => {}
        }
    }

    #[inline]
    fn delete_child(&mut self) {
        match &self.typ {
            ArtNodeType::Node4 => {}
            ArtNodeType::Node16 => {}
            ArtNodeType::Node48 => {}
            ArtNodeType::Node256 => {}
            _ => {}
        }
    }

    #[inline]
    fn grow(&mut self) {
        match &self.typ {
            ArtNodeType::Node4 => {
                let new_node = Node::new_node16(self.keys.concat());

                *self = new_node;
            }
            ArtNodeType::Node16 => {
                let new_node = Node::new_node48(self.keys.concat());

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
                let new_node = Node::new_node4(self.keys.concat());

                *self = new_node;
            }
            ArtNodeType::Node48 => {
                let new_node = Node::new_node16(self.keys.concat());

                *self = new_node;
            }
            ArtNodeType::Node256 => {
                let new_node = Node::new_node48(self.keys.concat());

                *self = new_node;
            }
            _ => {}
        }
    }

    #[inline]
    fn is_full(&self) -> bool {
        self.get_size() == self.max_size()
    }

    #[inline]
    fn is_less(&self) -> bool {
        self.get_size() < self.min_size()
    }

    #[inline]
    fn is_leaf(&self) -> bool {
        self.typ == ArtNodeType::Leaf
    }

    #[inline]
    fn get_count(&self) -> usize {
        self.children_count
    }

    #[inline]
    fn incr_count(&mut self) {
        self.children_count += 1;
    }

    #[inline]
    fn max_size(&self) -> usize {
        match &self.typ {
            ArtNodeType::Node4 => NODE4MAX,
            ArtNodeType::Node16 => NODE16MAX,
            ArtNodeType::Node48 => NODE48MAX,
            ArtNodeType::Node256 => NODE256MAX,
            _ => {}
        }
    }

    #[inline]
    fn min_size(&self) -> usize {
        match &self.typ {
            ArtNodeType::Node4 => NODE4MIN,
            ArtNodeType::Node16 => NODE16MIN,
            ArtNodeType::Node48 => NODE48MIN,
            ArtNodeType::Node256 => NODE256MIN,
            _ => {}
        }
    }
}
