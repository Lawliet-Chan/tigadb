use std::fs;
use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::path::Path;
use std::{u64, u8};

#[cfg(target_os = "unix")]
use std::os::unix::prelude::*;
#[cfg(target_os = "windows")]
use std::os::windows::prelude::*;

pub(crate) struct GroupLog {
    dir: &'static str,

    meta_files: Vec<File>,
    data_files: Vec<File>,

    // writing-file Index in Vec<LogFile>  and  file length
    // keeping file length just be recorded that Value_Bytes_Offset.
    /// (usize, usize) = (index, file_length) = (index, value_offset)

    /// index and file length of data_file writing
    dfw_idx_len: (usize, u64),

    // index and file length of meta_file writing
    mfw_idx_len: (usize, u64),

    limit_per_file: u64,
}

//TODO: use mmap to read file data.
impl GroupLog {
    #[inline]
    pub(crate) fn new(dir: &'static str, limit_per_file: u64) -> Self {
        fs::create_dir_all(dir).expect(format!("create data dir {} error", dir).as_str());
        let paths = fs::read_dir(dir).expect("find no data dir");
        let mut data_files: Vec<File> = Vec::new();
        let mut meta_files: Vec<File> = Vec::new();
        for path in paths {
            let p = path.unwrap().path();
            if p.is_file() {
                if !p.file_name().unwrap().to_str().contains(".meta") {
                    //data_files
                    let df =
                        File::create(p).expect(format!("recover data file {} error", p).as_str());
                    data_files.push(df);
                } else {
                    //compacting_files
                    let cf = File::create(p)
                        .expect(format!("recover compacting file {} error", p).as_str());
                    meta_files.push(cf);
                }
            }
        }
        data_files.sort();
        let d_idx = data_files.len();
        let d_len = data_files.last().unwrap().metadata().unwrap().len();

        meta_files.sort();
        let m_idx = meta_files.len();
        let m_len = meta_files.last().unwrap().metadata().unwrap().len();

        GroupLog {
            dir,
            data_files,
            meta_files,
            dfw_idx_len: (d_idx, d_len),
            mfw_idx_len: (m_idx, m_len),
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

    // (u8, u64, u64, u64) = (file_idx, offset, key_offset, length)
    #[inline]
    pub(crate) fn append_meta(&mut self, meta: (u8, u64, u64, u64), fsync: bool) -> io::Result<()> {
        let meta_u8 = Self::meta_to_bytes(meta);
        let mut f: &File;
        if self.mfw_idx_len.1 + meta_u8.len() as u64 <= self.limit_per_file
            && self.mfw_idx_len.0 > 0
        {
            f = self
                .meta_files
                .get_mut(self.mfw_idx_len.0)
                .ok_or(Err("Writing: find no meta file"))?;
        } else {
            f = &File::create(format!("{}/meta.{}", self.dir, self.mfw_idx_len.0 + 1).as_str())?;
        }
        let len = f.write(meta_u8)?;
        if fsync {
            f.sync_all()?;
        }
        Ok(())
    }

    // (u8, u64, u64) = (value_file_index, value_offset, value_length)
    #[inline]
    pub(crate) fn append_data(&mut self, data: &[u8], fsync: bool) -> io::Result<(u8, u64, u64)> {
        let mut f: &File;
        if self.dfw_idx_len.1 + data.len() as u64 <= self.limit_per_file && self.dfw_idx_len.0 > 0 {
            f = self
                .data_files
                .get_mut(self.dfw_idx_len.0)
                .ok_or(Err("writing: find no data file"))?;
        } else {
            f = &File::create(format!("{}/data.{}", self.dir, self.dfw_idx_len.0 + 1).as_str())?;
            self.dfw_idx_len.0 += 1;
            self.dfw_idx_len.1 = 0;
        }
        let len = f.write(data)?;
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
    pub(crate) fn write_meta(
        &mut self,
        meta: (u8, u64, u64, u64),
        pos: (u8, u64),
        fsync: bool,
    ) -> io::Result<()> {
        let bytes = Self::meta_to_bytes(meta);
        let mut f = self
            .meta_files
            .get_mut(pos.0 as usize)
            .ok_or(Err("Writing: find no meta file"))?;
        Self::write_at(f, bytes, pos.1)?;
        if fsync {
            f.sync_all()?;
        }
        Ok(())
    }

    #[inline]
    pub(crate) fn read_all_meta(&self) -> io::Result<[u8]> {
        Self::read_all(&self.meta_files)
    }

    #[inline]
    pub(crate) fn read_all_data(&self) -> io::Result<[u8]> {
        Self::read_all(&self.data_files)
    }

    #[inline]
    fn read_all(files: &Vec<File>) -> io::Result<[u8]> {
        let ref mut buf: Vec<u8> = Vec::new();
        let mut it = files.iter();
        let file = it.next();
        while let Some(mut f) = file {
            f.read_to_end(buf)?;
        }
        Ok(*buf.as_slice())
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

    #[inline]
    fn write_at(file: &mut File, buf: &[u8], offset: u64) -> io::Result<usize> {
        #[cfg(target_os = "unix")]
        {
            file.write_at(buf, offset)
        }

        #[cfg(target_os = "windows")]
        {
            file.seek_write(buf, offset)
        }
    }

    #[inline]
    fn meta_to_bytes(meta: (u8, u64, u64, u64)) -> &[u8] {
        let off_u8: [u8; 8] = meta.1.to_be_bytes;
        let koff_u8: [u8; 8] = meta.2.to_be_bytes;
        let len_u8: [u8; 8] = meta.3.to_be_bytes;
        [meta.0, off_u8, koff_u8, len_u8].concat().as_slice()
    }

    #[inline]
    fn bytes_to_meta(bytes: &[u8]) -> (u8, u64, u64, u64) {
        let (fdix_u8, left3) = bytes.split_at(1);
        let (off_u8, left2) = left3.split_at(8);
        let (k_off_u8, len_u8) = left2.split_at(8);
        let fidx = u8::from_be_bytes(*fdix_u8 as [u8; 1]);
        let off = u64::from_be_bytes(*off_u8 as [u8; 8]);
        let k_off = u64::from_be_bytes(*len_u8 as [u8; 8]);
        let len = u64::from_be_bytes(*len_u8 as [u8; 8]);
        (fidx, off, k_off, len)
    }
}
