const NODE4MIN: usize = 2;
const NODE4MAX: usize = 4;

const NODE16MIN: usize = 5;
const NODE16MAX: usize = 16;

const NODE48MIN: usize = 17;
const NODE48MAX: usize = 48;

const NODE256MIN: usize = 49;
const NODE256MAX: usize = 256;

const PREFIX_LEN: usize = 10;

struct Node {}

struct Node4 {
    keys: [char; 4],
    children: [Node; 4],
}

struct Node16 {
    keys: [char; 16],
    children: [Node; 16],
}

struct Node48 {
    keys: [char; 256],
    children: [Node; 48],
}

struct Node256 {
    children: [Node; 256],
}

fn new_node4() -> Node4 {}

fn new_node16() -> Node16 {}

fn new_node48() -> Node48 {}

fn new_node256() -> Node256 {}
