use crate::util::{bytes_to_u8, open_or_create_file, read_at, write_at};
use std::fs::File;
use std::io;

pub(crate) struct Wal {
    writing_file: usize,
    log_files: (File, File),
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
        let f1_stat = bytes_to_u8(f1_status_bytes);
        let f2_stat = bytes_to_u8(f2_status_bytes);
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
            log_files: (log_file_1, log_file_2),
            max_size_per_file,
        }
    }

    //pub(crate) fn append_data(&mut self) -> io::Result<()> {}
}

type FileStatus = u8;

const READ_ONLY: FileStatus = 0;
const WRITING: FileStatus = 1;
