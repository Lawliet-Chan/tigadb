use crate::group_logs::GroupLog;
use std::io;
use std::u64;

pub(crate) struct KV {
    key_log: GroupLog,
    value_log: GroupLog,
}

impl KV {
    #[inline]
    pub(crate) fn new(key_dir: &'static str, value_dir: &'static str, limit_per_file: u64) -> Self {
        KV {
            key_log: GroupLog::new(key_dir, limit_per_file),
            value_log: GroupLog::new(value_dir, limit_per_file),
        }
    }

    #[inline]
    pub(crate) fn write(
        &mut self,
        key: &[u8],
        value: &[u8],
        fsync: bool,
    ) -> io::Result<(u8, u64, u64)> {
        let value_pos = self.value_log.write(value, fsync)?;
        //let v_off: [u8;8] = value_pos.1.to_be_bytes;
        //let v_len: [u8;8] = value_pos.2.to_be_bytes;
        //let key_and_vpos = value_pos.0 + v_off + v_len + key;
        //self.key_log.write(key_and_vpos,fsync)
    }

    #[inline]
    pub(crate) fn read(&self, pos: (u8, u64, usize)) -> io::Result<[u8]> {}
}
