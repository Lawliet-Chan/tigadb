enum InnerNodeType {
    Node4,
    Node16,
    Node48,
    Node256,
}

impl InnerNodeType {
    fn max(self) -> usize {
        match self {
            InnerNodeType::Node4 => NODE4MAX,
            InnerNodeType::Node16 => NODE16MAX,
            InnerNodeType::Node48 => NODE48MAX,
            InnerNodeType::Node256 => NODE256MAX,
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

struct InnerNode {
    typ: NodeType,
    keys: [char],
    children: [Node],
    prefix: [char],
}

struct LeafNode {
    key: [u8],
    value: [u8],
}

impl Node {
    fn new_inner_node(typ: NodeType) -> Box<InnerNode> {
        Box::from(InnerNode {
            typ,
            keys: [char; typ.max()],
            children: [Node; typ.max()],
            prefix: [char; PREFIX_LEN],
        })
    }

    fn new_leaf_node(key: &[u8], value: &[u8]) -> Box<LeafNode> {
        Box::from(LeafNode {
            key: *key.to_owned(),
            value: *value.to_owned(),
        })
    }
}
