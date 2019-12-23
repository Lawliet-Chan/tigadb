use std::fs;
use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::path::Path;

#[cfg(target_os = "unix")]
use std::os::unix::prelude::*;
#[cfg(target_os = "windows")]
use std::os::windows::prelude::*;

pub(crate) struct GroupLog {
    dir: &'static str,
    files: Vec<LogFile>,

    // writing-file Index in Vec<LogFile>  and  file length
    // keeping file length just be recorded that Value_Bytes_Offset.
    // (usize, usize) = (index, file_length) = (index, value_offset)
    wf_idx_len: (usize, usize),
    limit_per_file: usize,
}

impl GroupLog {
    #[inline]
    pub(crate) fn new(dir: &'static str, limit_per_file: usize) -> Self {
        fs::create_dir_all(dir).expect(format!("create value dir {} error", dir).as_str());
        let paths = fs::read_dir(dir).expect("find no value dir");
        let mut files: Vec<LogFile> = Vec::new();
        for path in paths {
            let p = path.unwrap().path();
            if p.is_file() {
                let lf = LogFile::new(p).expect(format!("recover value log {} error", p).as_str());
                files.push(lf);
            }
        }
        files.sort();
        let idx = files.len();
        let len = files.last().unwrap().len();
        GroupLog {
            dir,
            files,
            wf_idx_len: (idx, len),
            limit_per_file,
        }
    }

    #[inline]
    pub(crate) fn read(&self, fidx: u8, offset: u64, len: usize) -> io::Result<[u8]> {
        let lf = self
            .files
            .get(fidx as usize)
            .ok_or(Err("reading: find no value-log file"))?;
        lf.read(offset, len)
    }

    // (u8, u64, usize) = (value_file_index, value_offset, value_length)
    #[inline]
    pub(crate) fn write(&mut self, value: &[u8], fsync: bool) -> io::Result<(u8, u64, usize)> {
        let mut lf: &LogFile;
        if self.wf_idx_len.1 + value.len() <= self.limit_per_file && self.wf_idx_len.0 > 0 {
            lf = self
                .files
                .get(self.wf_idx_len.0)
                .ok_or(Err("writing: find no value-log file"))?;
        } else {
            lf = &LogFile::new(format!("{}/value.{}", self.dir, self.wf_idx_len.0 + 1).as_str())?;
            self.wf_idx_len.0 += 1;
            self.wf_idx_len.1 = 0;
        }
        let len = lf.write(value, fsync)?;
        Ok((self.wf_idx_len.0 as u8, self.wf_idx_len.1 as u64, len))
    }

    #[inline]
    pub(crate) fn read_all(&self) -> io::Result<[u8]> {
        let mut buf: Vec<[u8]> = Vec::new();
        let mut it = self.files.iter();
        let f = it.next();
        while let Some(mut log_file) = f {
            let data = log_file.read_all()?;
            buf.push(data);
        }
        Ok(*buf.concat().as_slice())
    }

    #[inline]
    fn gc(&mut self) -> io::Result<()> {}
}

struct LogFile {
    path: &'static str,
    file: File,
}

impl LogFile {
    #[inline]
    fn new<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        let file = File::create(path)?;
        Ok(Self { path, file })
    }

    #[inline]
    fn read(&self, offset: u64, len: usize) -> io::Result<[u8]> {
        self.read_at(offset, len)
    }

    #[inline]
    fn write(&mut self, value: &[u8], fsync: bool) -> io::Result<usize> {
        let len = self.file.write(value)?;
        if fsync {
            self.file.sync_all()?;
        }
        Ok(len)
    }

    #[inline]
    fn len(&self) -> usize {
        self.file.metadata().unwrap().len() as usize
    }

    #[inline]
    fn read_all(&mut self) -> io::Result<[u8]> {
        let mut buf = Vec::new();
        self.file.read_to_end(buf)?;
        buf.as_slice()
    }

    #[inline]
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

    #[inline]
    fn write_at(&mut self, buf: &[u8], offset: u64) -> io::Result<usize> {
        #[cfg(target_os = "unix")]
        {
            self.file.write_at(buf, offset)
        }

        #[cfg(target_os = "windows")]
        {
            self.file.seek_write(buf, offset)
        }
    }
}
