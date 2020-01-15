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
    /// (usize, usize) = (index, file_length) = (index, value_offset)

    /// index and file length of data_file writing
    dfw_idx_len: (usize, u64),

    /// index and file length of cpt_file writing
    cfw_idx_len: (usize, u64),

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
        let d_idx = data_files.len();
        let d_len = data_files.last().unwrap().metadata().unwrap().len();

        cpt_files.sort();
        let c_idx = cpt_files.len();
        let c_len = cpt_files.last().unwrap().metadata().unwrap().len();
        GroupLog {
            dir,
            data_files,
            cpt_files,
            dfw_idx_len: (d_idx, d_len),
            cfw_idx_len: (c_idx, c_len),
            limit_per_file,
        }
    }

    #[inline]
    pub(crate) fn read_data(&self, fidx: u8, offset: u64, len: usize) -> io::Result<[u8]> {
        let f = self
            .data_files
            .get(fidx as usize)
            .ok_or(Err("reading: find no data file"))?;
        Self::read_file(f, offset, len)
    }

    // (u8, u64, u64) = (value_file_index, value_offset, value_length)
    #[inline]
    pub(crate) fn write_data(&mut self, value: &[u8], fsync: bool) -> io::Result<(u8, u64, u64)> {
        let mut f: &File;
        if self.dfw_idx_len.1 + value.len() as u64 <= self.limit_per_file && self.dfw_idx_len.0 > 0
        {
            f = self
                .data_files
                .get(self.dfw_idx_len.0)
                .ok_or(Err("writing: find no data file"))?;
        } else {
            f = &File::create(format!("{}/data.{}", self.dir, self.dfw_idx_len.0 + 1).as_str())?;
            self.dfw_idx_len.0 += 1;
            self.dfw_idx_len.1 = 0;
        }
        let len = f.write(value)?;
        if fsync {
            f.sync_all()?;
        }
        Ok((
            self.dfw_idx_len.0 as u8,
            self.dfw_idx_len.1 as u64,
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
    pub(crate) fn write_cpt(&mut self, pos: Vec<(u8, u64, u64)>) -> io::Result<(u8, u64, u64)> {
        let mut cpt_data = Vec::new();
        let mut iter = pos.iter();
        while let Some(po) = iter.next() {
            let idx = po.0;
            let offset = po.1;
            let len = po.2 as usize;
            if let Some(data_file) = self.data_files.get(idx as usize) {
                let data = Self::read_file(data_file, offset, len)?;
                cpt_data.append(data);
            }
        }
        if !cpt_data.is_empty() {
            let mut f: &File;
            if self.cfw_idx_len.1 + cpt_data.len() as u64 <= self.limit_per_file
                && self.cfw_idx_len.0 > 0
            {
                f = self
                    .cpt_files
                    .get(self.cfw_idx_len.0)
                    .ok_or(Err("compacting: find no compacting file"))?;
            } else {
                f = &File::create(format!("{}/cpt.{}", self.dir, self.cfw_idx_len.0 + 1).as_str())?;
                self.cfw_idx_len.0 += 1;
                self.cfw_idx_len.1 = 0;
            }
            let len = f.write(cpt_data)?;
            f.sync_all()?;
        }
        Ok((
            self.cfw_idx_len.0 as u8,
            self.cfw_idx_len.1 as u64,
            len as u64,
        ))
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
