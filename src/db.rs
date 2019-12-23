use crate::art::Node;
use crate::option::Option;
use std::sync::atomic::AtomicUsize;
use std::sync::Arc;

pub struct DB {
    opt: Option,

    // write-transaction id
    // I give each write-txn an ID by txn_id
    // and spawn one thread to execute all concurrent writing transactions.
    txn_id: AtomicUsize,

    tree: Node,
}

impl DB {
    pub fn new(opt: Option) -> DB {
        DB {
            opt,
            txn_id: AtomicUsize::new(0),
        }
    }
}
