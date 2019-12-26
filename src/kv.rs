use crate::group_logs::GroupLog;
use std::io;
use std::u64;

pub(crate) struct KV {
    meta_log: GroupLog,
    kv_log: GroupLog,
}

impl KV {
    #[inline]
    pub(crate) fn new(key_dir: &'static str, value_dir: &'static str, limit_per_file: u64) -> Self {
        KV {
            meta_log: GroupLog::new(key_dir, limit_per_file),
            kv_log: GroupLog::new(value_dir, limit_per_file),
        }
    }

    #[inline]
    pub(crate) fn write(
        &mut self,
        key: &[u8],
        value: &[u8],
        fsync: bool,
    ) -> io::Result<(u8, u64, u64)> {
        let dividing_point = key.len() as u64;
        let kv_data = [*key, *value].concat().as_slice();
        let kv_pos = self.kv_log.write(kv_data, fsync)?;

        let kv_offset_u8: [u8; 8] = kv_pos.1.to_be_bytes;
        let kv_len_u8: [u8; 8] = kv_pos.2.to_be_bytes;
        let dividing_point_u8: [u8; 8] = dividing_point.to_be_bytes;
        let metadata = [kv_pos.0, kv_offset_u8, kv_len_u8, dividing_point_u8]
            .concat()
            .as_slice();
        self.meta_log.write(metadata, fsync)?;
        kv_pos.1 += dividing_point;
        Ok(kv_pos)
    }

    #[inline]
    pub(crate) fn read(&self, pos: (u8, u64, usize)) -> io::Result<[u8]> {}
}
