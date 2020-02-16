use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::ops::Bound::Included;
use std::path::Path;
use std::u32;
use std::u8;

const KV_POS_SIZE: usize = 10;
const BLOCK_SIZE: usize = 512;
const MAX_KV_SIZE: usize = BLOCK_SIZE * u8::max_value();
const MAX_BLOCK_ID: usize = u32::max_value();

pub(crate) struct Storage {
    kv_pos: Vec<u8>,
    // store kv_pos
    meta_file: File,

    data_file: File,

    wal_file: File,

    // start_block_id --> blocks
    pending_blocks_start: BTreeMap<BlockId, Blocks>,
    // end_block_id --> blocks
    pending_blocks_end: BTreeMap<BlockId, Blocks>,

    pending_blocks_set: BTreeSet<Blocks>,
}

impl Storage {
    pub(crate) fn new(
        data_fpath: &'static str,
        meta_fpath: &'static str,
        wal_fpath: &'static str,
    ) -> Self {
        let meta_file = Self::open_or_create_file(meta_fpath);
        let data_file = Self::open_or_create_file(data_fpath);
        let wal_file = Self::open_or_create_file(wal_fpath);

        let kv_pos = Vec::new();
        let pending_blocks_set = BTreeSet::new();
        let pending_blocks_start = BTreeMap::new();
        let pending_blocks_end = BTreeMap::new();

        Self {
            kv_pos,
            meta_file,
            data_file,
            wal_file,
            pending_blocks_start,
            pending_blocks_end,
            pending_blocks_set,
        }
    }

    fn open_or_create_file(fpath: &'static str) -> File {
        if Path::new(fpath).exists() {
            File::open(fpath).expect(format!("open file {} error", fpath).as_str())
        } else {
            File::create(fpath).expect(format!("create file {} error", fpath).as_str())
        }
    }

    pub(crate) fn read_kv(&self, kv_pos: KVpos) -> io::Result<Vec<u8>> {
        let offset = (kv_pos.blocks.start_block_id * BLOCK_SIZE + kv_pos.value_pos as u32) as u64;
        let len = (kv_pos.kv_size - kv_pos.value_pos) as usize;
        read_at(&self.data_file, offset, len)
    }

    pub(crate) fn write_kv(&mut self, multi_data: Vec<Vec<u8>>) -> io::Result<()> {}

    pub(crate) fn write_meta(&mut self, multi_meta: Vec<KVpos>) -> io::Result<()> {}

    pub(crate) fn delete_kv(&mut self, kv_pos: KVpos) -> io::Result<()> {}

    pub(crate) fn append_wal(&mut self, multi_data: Vec<Vec<u8>>) -> io::Result<()> {}

    pub(crate) fn truncate_wal(&mut self, offset: usize) -> io::Result<()> {}

    fn insert_pending_blocks(&mut self, blocks: &mut Blocks) {
        let first = blocks.first_block_id();
        let last = blocks.last_block_id();
        if let Some(pblocks) = self.pending_blocks_end.get(&(first - 1)) {
            blocks.merge_to_head(pblocks);
            self.pending_blocks_end.remove(&(first - 1));
            self.pending_blocks_set.remove(pblocks);
        }
        if let Some(pblocks) = self.pending_blocks_start.get(&(last + 1)) {
            blocks.merge_to_tail(pblocks);
            self.pending_blocks_start.remove(&(last + 1));
            self.pending_blocks_set.remove(pblocks);
        }
        self.pending_blocks_set.insert(blocks.clone());
        self.pending_blocks_start
            .insert(blocks.first_block_id(), blocks.clone());
        self.pending_blocks_end
            .insert(blocks.last_block_id(), blocks.to_owned());
    }

    fn take_pending_blocks(&mut self, data_size: usize) -> Option<&Blocks> {
        let needed_blocks = (data_size / BLOCK_SIZE) as u8;
        let range_blocks = self
            .pending_blocks_set
            .range((Included(needed_blocks), Included(MAX_KV_SIZE)));
        range_blocks.min()
    }

    fn remove_pending_blocks(&mut self, blocks: Blocks) {
        self.pending_blocks_set.remove(&blocks);
        self.pending_blocks_start.remove(&blocks.first_block_id());
        self.pending_blocks_end.remove(&blocks.last_block_id());
    }
}

pub(crate) struct KVpos {
    blocks: Blocks,
    value_pos: u16,
    kv_size: u16,
}

type BlockId = u32;
type BlocksLen = u8;

// consecutive blocks
#[derive(Clone)]
struct Blocks {
    start_block_id: BlockId,
    block_count: BlocksLen,
}

impl Blocks {
    fn new(start_block_id: BlockId) -> Self {
        Self {
            start_block_id,
            block_count: 0,
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

fn read_at(file: &File, offset: u64, len: usize) -> io::Result<Vec<u8>> {
    let buf = &mut Vec::with_capacity(len);
    #[cfg(target_os = "unix")]
    {
        file.read_at(buf, offset)
    }

    #[cfg(target_os = "windows")]
    {
        file.seek_read(buf, offset)
    }
    Ok(buf.to_vec())
}

fn write_at(file: &mut File, buf: &mut [u8], offset: u64) -> io::Result<usize> {
    #[cfg(target_os = "unix")]
    {
        file.write_at(buf, offset)?
    }

    #[cfg(target_os = "windows")]
    {
        file.seek_write(buf, offset)?
    }
}
