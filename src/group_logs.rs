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
    data_files: Vec<File>,

    //compacting files
    cpt_files: Vec<File>,

    // writing-file Index in Vec<LogFile>  and  file length
    // keeping file length just be recorded that Value_Bytes_Offset.
    // (usize, usize) = (index, file_length) = (index, value_offset)
    wf_idx_len: (usize, u64),
    limit_per_file: u64,
}

//TODO: use mmap to read file data.
impl GroupLog {
    #[inline]
    pub(crate) fn new(dir: &'static str, limit_per_file: u64) -> Self {
        fs::create_dir_all(dir).expect(format!("create data dir {} error", dir).as_str());
        let paths = fs::read_dir(dir).expect("find no data dir");
        let mut data_files: Vec<File> = Vec::new();
        let mut cpt_files: Vec<File> = Vec::new();
        for path in paths {
            let p = path.unwrap().path();
            if p.is_file() {
                if !p.file_name().unwrap().to_str().contains(".cpt") {
                    //data_files
                    let df =
                        File::create(p).expect(format!("recover data file {} error", p).as_str());
                    data_files.push(df);
                } else {
                    //compacting_files
                    let cf = File::create(p)
                        .expect(format!("recover compacting file {} error", p).as_str());
                    cpt_files.push(cf);
                }
            }
        }
        data_files.sort();
        cpt_files.sort();
        let idx = data_files.len();
        let len = data_files.last().unwrap().len();
        GroupLog {
            dir,
            data_files,
            cpt_files,
            wf_idx_len: (idx, len),
            limit_per_file,
        }
    }

    #[inline]
    pub(crate) fn read(&self, fidx: u8, offset: u64, len: usize) -> io::Result<[u8]> {
        let f = self
            .data_files
            .get(fidx as usize)
            .ok_or(Err("reading: find no data file"))?;
        Self::read_file(f, offset, len)
    }

    // (u8, u64, u64) = (value_file_index, value_offset, value_length)
    #[inline]
    pub(crate) fn write(&mut self, value: &[u8], fsync: bool) -> io::Result<(u8, u64, u64)> {
        let mut f: &File;
        if self.wf_idx_len.1 + value.len() as u64 <= self.limit_per_file && self.wf_idx_len.0 > 0 {
            f = self
                .data_files
                .get(self.wf_idx_len.0)
                .ok_or(Err("writing: find no data file"))?;
        } else {
            f = &File::create(format!("{}/data.{}", self.dir, self.wf_idx_len.0 + 1).as_str())?;
            self.wf_idx_len.0 += 1;
            self.wf_idx_len.1 = 0;
        }
        let len = f.write(value)?;
        if fsync {
            f.sync_all()?;
        }
        Ok((
            self.wf_idx_len.0 as u8,
            self.wf_idx_len.1 as u64,
            len as u64,
        ))
    }

    #[inline]
    pub(crate) fn read_all(&self) -> io::Result<[u8]> {
        let ref mut buf: Vec<u8> = Vec::new();
        let mut it = self.data_files.iter();
        let file = it.next();
        while let Some(mut f) = file {
            f.read_to_end(buf)?;
        }
        Ok(*buf.as_slice())
    }

    #[inline]
    fn compact(&mut self, pos: Vec<(u8, u64, u64)>) -> io::Result<()> {
        let mut iter = pos.iter();
        while let Some(po) = iter.next() {}
        Ok(())
    }

    #[inline]
    fn read_file(file: &File, offset: u64, len: usize) -> io::Result<[u8]> {
        let buf = &mut [u8; len];
        #[cfg(target_os = "unix")]
        {
            file.read_at(buf, offset)
        }

        #[cfg(target_os = "windows")]
        {
            file.seek_read(buf, offset)
        }
    }
}
