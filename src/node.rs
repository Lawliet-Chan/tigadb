enum Node {
    Node4 {
        key: [char; 4],
        children: [Node; 4],
    },
    Node16 {
        key: [char; 16],
        children: [Node; 16],
    },
    Node48 {
        key: [char; 256],
        children: [Node; 48],
    },
    Node256 {
        children: [Node; 256],
    },
    Leaf,
}

impl Node {}
