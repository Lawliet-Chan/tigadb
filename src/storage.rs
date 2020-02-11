use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::io;
use std::io::prelude::*;

const KV_POS_SIZE: usize = 10;
const BLOCK_SIZE: usize = 512;

pub(crate) struct Storage {
    kv_pos: Vec<u8>,
    meta_file: File,

    kv_data: Vec<u8>,
    data_file: File,

    pending_blocks: HashMap<u32, Blocks>,
}

impl Storage {
    pub(crate) fn new(data_fpath: &'static str, meta_fpath: &'static str) -> Self {}
}

// consecutive blocks
struct Blocks(Vec<Block>);

impl Blocks {
    fn new() -> Self {
        Self(Vec::new())
    }

    fn start_block_id(&self) -> Option<u32> {
        if let Some(block) = self.0.first() {
            Some(block.id)
        } else {
            None
        }
    }

    fn len(&self) -> usize{
        self.0.len()
    }
}

type BlockId = u32;

struct Block {
    id: BlockId,
    kv_data: Vec<u8>,
}

pub(crate) struct KVpos {
    block_id: BlockId,
    over_blocks: u8,
    key_pos: u16,
    value_pos: u16,
    kv_size: u16,
}
