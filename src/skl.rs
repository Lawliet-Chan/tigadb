struct SkipList<K> {
    height: u16,
    head: Box<Node<K>>,

    node_cap: usize,
}

impl<K> SkipList<K>
where
    K: AsRef<[u8]>,
{
    #[inline]
    fn new(node_cap: usize) -> SkipList<K> {
        SkipList {
            height: 1,
            head: Box::new(Node::new(1, node_cap)),
            node_cap,
        }
    }

    #[inline]
    fn insert(&mut self, key: K) {}

    #[inline]
    fn get(&self, key: K) -> Option<Box<Node<K>>> {}

    #[inline]
    fn get_mut(&mut self, key: K) -> Option<Box<Node<K>>> {}
}

struct Node<K> {
    keys: Vec<K>,
    level: u16,
    next: Option<Box<Node<K>>>,
    prev: Option<Box<Node<K>>>,
    below: Option<Box<Node<K>>>,
}

impl<K> Node<K>
where
    K: AsRef<[u8]>,
{
    #[inline]
    fn new(level: u16, cap: usize) -> Self {
        Node {
            keys: Vec::with_capacity(cap),
            level,
            next: None,
            prev: None,
            below: None,
        }
    }

    #[inline]
    fn insert(&mut self, key: K) {}

    #[inline]
    fn len(&self) -> usize {
        self.keys.len()
    }

    #[inline]
    fn next(&self) -> Option<Box<Node<K>>> {
        self.next
    }

    #[inline]
    fn next_mut(&mut self) -> Option<Box<Node<K>>> {
        self.next
    }

    #[inline]
    fn prev(&self) -> Option<Box<Node<K>>> {
        self.prev
    }

    #[inline]
    fn prev_mut(&mut self) -> Option<Box<Node<K>>> {
        self.prev
    }
}
