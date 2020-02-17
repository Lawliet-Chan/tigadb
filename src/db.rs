use crate::art::ArtTree;
use crate::option::Option;
use std::sync::atomic::AtomicUsize;
use std::sync::{Arc, RwLock};
use std::time::SystemTime;

pub struct DB<'a> {
    opt: Option,

    // write-transaction id
    // I give each write-txn an ID by txn_id
    // and spawn one thread to execute all concurrent writing transactions.
    txn_id: AtomicUsize,

    // When data has been stored in disk and apply into ART-tree, it would be recorded two timestamps
    // because now the data can be read for user but the data maybe not in ART-tree yet.
    /// commit_ts is the last timestamp when the data are stored in disk.
    commit_ts: SystemTime,

    /// apply_ts is the last timestamp when the data are applied into ART-tree.
    apply_ts: SystemTime,
    // When apply_ts < reading_request_ts <= commit_ts and reading_key is in key_cache,
    // the reading ops will wait until reading_request_ts reaches apply_ts.
    // Otherwise, just read in ART-tree.

    // key_cache is already in disk and going to apply into ART-tree.
    key_cache: Arc<Vec<u8>>,

    tree: ArtTree<'a>,
    //disk: Arc<RwLock<KV>>,
}

impl<'a> DB<'a> {
    pub fn new(opt: Option) -> DB<'a> {
        let now = SystemTime::now();
        DB {
            opt,
            txn_id: AtomicUsize::new(0),
            commit_ts: now,
            key_cache: Arc::new(Vec::new()),
            apply_ts: now,
            //disk: KV::new(opt.meta_dir, opt.kv_dir, opt.limit_per_file),
            tree: ArtTree::default(),
        }
    }
}
