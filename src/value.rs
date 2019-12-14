use std::fs::File;
use std::io;
use std::io::{prelude::*, BufReader};

struct ValueLog {
    files: Vec<LogFile>,
    limit_per_file: u64,
}

struct LogFile {
    file: File,
}

impl ValueLog {
    fn new(limit_per_file: u64) -> Self {
        ValueLog {
            files: Vec::new(),
            limit_per_file,
        }
    }

    fn read(&self, fidx: usize, offset: usize, len: usize) -> io::Result<()> {}

    fn write(&mut self, fidx: usize) -> io::Result<()> {
        write!()
    }
}
