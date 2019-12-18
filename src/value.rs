use std::fs::File;
use std::io;
use std::io::{prelude::*, BufReader};

#[cfg(target_os = "unix")]
use std::os::unix::prelude::*;
#[cfg(target_os = "windows")]
use std::os::windows::prelude::*;

struct ValueLog {
    files: Vec<LogFile>,
    limit_per_file: u64,
}

impl<V: AsRef<[u8]>> ValueLog {
    pub(crate) fn new(limit_per_file: u64) -> Self {
        ValueLog {
            files: Vec::new(),
            limit_per_file,
        }
    }

    pub(crate) fn read(&self, fidx: usize, offset: u64, len: usize) -> io::Result<([u8])> {
        self.files.get(fidx).ok_or(Err("find no log file"));
    }

    pub(crate) fn write(&mut self, fidx: usize, value: V) -> io::Result<(usize)> {
        let mut lf = self.files.get(fidx).get_or_insert_with(LogFile::new());
        lf.write(value)
    }
}

struct LogFile {
    file: File,
}

impl<V: AsRef<[u8]>> LogFile {
    fn new() -> Self {}

    fn read(&self, offset: u64, len: usize) -> io::Result<([u8])> {
        self.read_at(offset, len)
    }

    fn write(&mut self, value: V) -> io::Result<(usize)> {
        self.file.write(value.as_ref())
    }

    fn len(&self) -> u64 {
        self.file.metadata().unwrap().len()
    }

    fn read_at(&self, offset: u64, len: usize) -> io::Result<[u8]> {
        let buf = &mut [u8; len];
        #[cfg(target_os = "unix")]
        {
            self.file.read_at(buf, offset)
        }

        #[cfg(target_os = "windows")]
        {
            self.file.seek_read(buf, offset)
        }
    }
}
