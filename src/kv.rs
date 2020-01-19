use crate::group_logs::GroupLog;
use std::io;
use std::thread;

pub(crate) struct KV {
    kv_store: GroupLog,
    cpt_store: GroupLog,
}

impl KV {
    #[inline]
    pub(crate) fn new(kv_dir: &'static str, cpt_dir: &'static str, limit_per_file: u64) -> Self {
        let kv_store = GroupLog::new(kv_dir, limit_per_file);
        let cpt_store = GroupLog::new(cpt_dir, limit_per_file);
        let kv = KV {
            kv_store,
            cpt_store,
        };
        thread::spawn(|| {
            &kv.gc();
        });
        kv
    }

    // (u8, u64, u64) = (kv_log_index, kv_offset, kv_length)
    #[inline]
    pub(crate) fn read(&self, value_pos: (u8, u64, u64)) -> io::Result<Vec<u8>> {
        self.kv_store.read_data(value_pos)
    }

    // (u8, u64, u64) = (kv_log_index, value_offset, value_length)
    #[inline]
    pub(crate) fn write(&mut self, data: &[u8], fsync: bool) -> io::Result<(u8, u64, u64)> {
        self.kv_store.append_data(data, fsync)
    }

    // when commit deleting data, append (u8, u64, u64, 0)
    #[inline]
    pub(crate) fn commit(&mut self, kv_pos: (u8, u64, u64, u64), fsync: bool) -> io::Result<()> {
        self.kv_store.append_meta(kv_pos, fsync)
    }

    #[inline]
    fn gc(&self) {
        match self.kv_store.read_all_meta() {
            Ok(metadata) => {}
            Err(e) => {}
        };
    }
}
