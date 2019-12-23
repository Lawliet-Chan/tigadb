use crate::group_logs::GroupLog;
use std::io;

pub(crate) struct KV {
    key_log: GroupLog,
    value_log: GroupLog,
}

impl KV {
    #[inline]
    pub(crate) fn new(
        key_dir: &'static str,
        value_dir: &'static str,
        limit_per_file: usize,
    ) -> Self {
        KV {
            key_log: GroupLog::new(key_dir, limit_per_file),
            value_log: GroupLog::new(value_dir, limit_per_file),
        }
    }

    #[inline]
    pub(crate) fn write_key(&mut self) {}

    #[inline]
    pub(crate) fn write_value(&mut self) {}

    #[inline]
    pub(crate) fn read_key(&self, pos: (u8, u64, usize)) -> io::Result<[u8]> {}

    #[inline]
    pub(crate) fn read_value(&self, pos: (u8, u64, usize)) -> io::Result<[u8]> {}
}
