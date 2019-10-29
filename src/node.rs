enum NodeType {
    Node4,
    Node16,
    Node48,
    Node256,
    Leaf,
}

impl NodeType {
    fn max(self) -> usize {
        match self {
            NodeType::Node4 => NODE4MAX,
            NodeType::Node16 => NODE16MAX,
            NodeType::Node48 => NODE48MAX,
            NodeType::Node256 => NODE256MAX,
            NodeType::Leaf => 0,
        }
    }
}

const NODE4MIN: usize = 2;
const NODE4MAX: usize = 4;

const NODE16MIN: usize = 5;
const NODE16MAX: usize = 16;

const NODE48MIN: usize = 17;
const NODE48MAX: usize = 48;

const NODE256MIN: usize = 49;
const NODE256MAX: usize = 256;

const PREFIX_LEN: usize = 10;

struct Node {
    typ: NodeType,
    key: [char],
    children: [Node],
    prefix: [char],
}

impl Node {
    pub fn new(typ: NodeType) -> Box<Node> {
        Box::from(Node {
            typ,
            key: [char; typ.max()],
            children: [Node; typ.max()],
            prefix: [char; PREFIX_LEN],
        })
    }
}
