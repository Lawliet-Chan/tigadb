use crate::util::{open_or_create_file, read_at};
use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet};
use std::fs::File;
use std::io;
use std::ops::Bound::Included;
use std::u32;
use std::u8;

const MAX_KV_SIZE: usize = BLOCK_SIZE as usize * u8::max_value() as usize;
const MAX_BLOCK_ID: BlockId = u32::max_value();

pub(crate) struct Storage {
    kv_pos: Vec<u8>,
    // store kv_pos
    meta_file: File,

    min_blocks_id_can_use: BlockId,
    data_file: File,

    // start_block_id --> blocks
    chink_blocks_start: BTreeMap<BlockId, Blocks>,
    // end_block_id --> blocks
    chink_blocks_end: BTreeMap<BlockId, Blocks>,
    // the bool True means that blocks can be read or written directly.
    // False means that kv data which in this blocks is written into other blocks BUT NOT COMMITTED yet.
    chink_blocks: BTreeMap<Blocks, bool>,
}

impl Storage {
    pub(crate) fn new(data_fpath: &'static str, meta_fpath: &'static str) -> Self {
        let meta_file = open_or_create_file(meta_fpath);
        let data_file = open_or_create_file(data_fpath);

        let kv_pos = Vec::new();
        let chink_blocks = BTreeMap::new();
        let chink_blocks_start = BTreeMap::new();
        let chink_blocks_end = BTreeMap::new();

        Self {
            kv_pos,
            meta_file,
            min_blocks_id_can_use: 0,
            data_file,
            chink_blocks_start,
            chink_blocks_end,
            chink_blocks,
        }
    }

    pub(crate) fn read_kv(&self, kv_pos: KVpos) -> io::Result<Vec<u8>> {
        let offset =
            (kv_pos.blocks.start_block_id * BLOCK_SIZE as u32 + kv_pos.value_pos as u32) as u64;
        let len = (kv_pos.kv_size - kv_pos.value_pos) as usize;
        read_at(&self.data_file, offset, len)
    }

    //pub(crate) fn write_kv(&mut self, multi_data: Vec<Vec<u8>>) -> io::Result<()> {}

    //pub(crate) fn write_meta(&mut self, multi_meta: Vec<KVpos>) -> io::Result<()> {}

    //pub(crate) fn delete_kv(&mut self, kv_pos: KVpos) -> io::Result<()> {}

    fn alloc_blocks(&mut self, needed_blocks: BlocksLen) -> Option<Blocks> {
        let chink_blocks = self.take_free_chink_blocks(needed_blocks);
        if let Some(blocks) = chink_blocks {
            Some(*blocks)
        } else {
            if needed_blocks as BlockId + self.min_blocks_id_can_use > MAX_BLOCK_ID {
                return None;
            }
            let new_blocks = Blocks::new(self.min_blocks_id_can_use, needed_blocks);
            // self.min_blocks_id_can_use += needed_blocks as BlockId;
            Some(new_blocks)
        }
    }

    // When update or delete KV, disk will make chink blocks.
    fn insert_chink_blocks(&mut self, blocks: &mut Blocks) {
        let first = blocks.first_block_id();
        let last = blocks.last_block_id();
        if let Some(pblocks) = self.chink_blocks_end.remove(&(first - 1)) {
            blocks.merge_to_head(&pblocks);
            self.chink_blocks.remove(&pblocks);
        }
        if let Some(pblocks) = self.chink_blocks_start.remove(&(last + 1)) {
            blocks.merge_to_tail(&pblocks);
            self.chink_blocks.remove(&pblocks);
        }
        self.chink_blocks.insert(blocks.clone(), true);
        self.chink_blocks_start
            .insert(blocks.first_block_id(), blocks.clone());
        self.chink_blocks_end
            .insert(blocks.last_block_id(), blocks.to_owned());
    }

    fn take_free_chink_blocks(&self, needed_blocks: BlocksLen) -> Option<&Blocks> {
        let mut it = self.chink_blocks.iter();
        while let Some(chink_blocks) = it.next() {
            if chink_blocks.0.count() >= needed_blocks && *chink_blocks.1 {
                return Some(chink_blocks.0);
            }
        }
        None
    }

    fn remove_chink_blocks(&mut self, blocks: Blocks) {
        self.chink_blocks.remove(&blocks);
        self.chink_blocks_start.remove(&blocks.first_block_id());
        self.chink_blocks_end.remove(&blocks.last_block_id());
    }
}

const KV_POS_SIZE: usize = 9;

pub(crate) struct KVpos {
    blocks: Blocks,
    value_pos: u16,
    kv_size: u16,
}

const BLOCK_SIZE: usize = 512;

type BlockId = u32;
type BlocksLen = u8;

// consecutive blocks
#[derive(Clone, Copy, Eq, PartialOrd, PartialEq, Debug)]
struct Blocks {
    start_block_id: BlockId,
    block_count: BlocksLen,
}

impl Blocks {
    fn new(start_block_id: BlockId, block_count: BlocksLen) -> Self {
        Self {
            start_block_id,
            block_count,
        }
    }

    fn first_block_id(&self) -> BlockId {
        self.start_block_id
    }

    fn last_block_id(&self) -> BlockId {
        self.start_block_id + self.block_count as BlockId
    }

    fn count(&self) -> BlocksLen {
        self.block_count
    }

    fn merge_to_tail(&mut self, blocks: &Blocks) {
        self.block_count += blocks.block_count;
    }

    fn merge_to_head(&mut self, blocks: &Blocks) {
        self.start_block_id = blocks.start_block_id;
        self.block_count += blocks.block_count;
    }
}

impl Ord for Blocks {
    fn cmp(&self, other: &Self) -> Ordering {
        self.block_count.cmp(&other.block_count)
    }
}
