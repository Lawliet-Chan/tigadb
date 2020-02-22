use crate::storage::KVpos;
use crate::util::{bytes_to_u8, open_or_create_file, read_at, u64_to_bytes, write_at};
use std::fs::File;
use std::io;
use std::io::Write;

pub(crate) struct Wal {
    writing_file: usize,
    log_files: [File; 2],
    max_size_per_file: usize,
}

impl Wal {
    pub(crate) fn new(f1: &'static str, f2: &'static str, max_size_per_file: usize) -> Self {
        let mut log_file_1 = open_or_create_file(f1);
        let log_file_2 = open_or_create_file(f2);
        let f1_status_bytes = read_at(&log_file_1, 0, 1)
            .expect(format!("read FileStatus from {} error", f1).as_str());
        let f2_status_bytes = read_at(&log_file_2, 0, 1)
            .expect(format!("read FileStatus from {} error", f2).as_str());
        let f1_stat = bytes_to_u8(f1_status_bytes.as_slice());
        let f2_stat = bytes_to_u8(f2_status_bytes.as_slice());
        let writing_file;
        if f1_stat == WRITING {
            writing_file = 0
        } else if f1_stat == READ_ONLY && f2_stat == READ_ONLY {
            write_at(&mut log_file_1, &mut [WRITING], 0)
                .expect(format!("write WRITING status into {} error", f1).as_str());
            writing_file = 0;
        } else {
            writing_file = 1
        }

        Wal {
            writing_file,
            log_files: [log_file_1, log_file_2],
            max_size_per_file,
        }
    }

    pub(crate) fn append_wal(
        &mut self,
        multi_old_kvpos: Vec<KVpos>,
        mut data: Vec<u8>,
        fsync: bool,
    ) -> io::Result<()> {
        let mut old_kvpos_bytes = Vec::new();
        let mut iter = multi_old_kvpos.iter();
        while let Some(old_kvpos) = iter.next() {
            let mut bytes = old_kvpos.to_bytes();
            old_kvpos_bytes.append(&mut bytes);
        }
        let old_kvpos_bytes_len = old_kvpos_bytes.len() as u64;
        let data_len = data.len() as u64;
        let mut bytes_to_append = Vec::new();
        bytes_to_append.append(&mut u64_to_bytes(old_kvpos_bytes_len));
        bytes_to_append.append(&mut u64_to_bytes(data_len));
        bytes_to_append.append(&mut old_kvpos_bytes);
        bytes_to_append.append(&mut data);
        self.log_files[self.writing_file].write_all(bytes_to_append.as_slice())?;
        if fsync {
            self.log_files[self.writing_file].sync_all()?;
        }
        Ok(())
    }

    pub(crate) fn truncate_wal(&mut self) -> io::Result<()> {
        self.log_files[self.writing_file].set_len(0)
    }
}

type FileStatus = u8;

const READ_ONLY: FileStatus = 0;
const WRITING: FileStatus = 1;
