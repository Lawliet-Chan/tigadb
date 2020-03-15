
/*
  The wal is hold on  and the development pause.
*/
use crate::storage::KVpos;
use crate::util::{
    bytes_to_u64, bytes_to_u8, open_or_create_file, read_at, u64_to_bytes, write_at,
};
use std::fs::File;
use std::io;
use std::io::{Read, Write};
use std::thread;

pub(crate) struct Wal {
    writing_file: LogFile,
    read_only_file: LogFile,
    max_size_per_file: u64,
}

impl Wal {
    pub(crate) fn new(f1: &'static str, f2: &'static str, max_size_per_file: u64) -> Self {
        let mut lf1 = LogFile::new(f1);
        let lf2 = LogFile::new(f2);
        let mut wf = lf1.0;
        let mut rf = lf2.0;

        let lf1_state = lf1.1;
        let lf2_state = lf2.1;
        if lf1_state == READ_ONLY && lf2_state == READ_ONLY {
            wf.set_writing_state();
        } else if lf2_state == WRITING {
            std::mem::swap(&mut wf, &mut rf);
        }

        Wal {
            writing_file: wf,
            read_only_file: rf,
            max_size_per_file,
        }
    }

    pub(crate) fn recover(&mut self, last_ckpt: u64) -> io::Result<Vec<(Vec<KVpos>, Vec<u8>)>> {
        let mut result = Vec::new();
        let wf_last_ckpt = self.writing_file.get_last_ckpt();
        let rf_last_ckpt = self.read_only_file.get_last_ckpt();
        if last_ckpt == wf_last_ckpt {
            return Ok(result);
        }
        if last_ckpt < rf_last_ckpt {
            let mut rf_result = self.read_only_file.recover(last_ckpt)?;
            result.append(&mut rf_result);
        }
        let mut wf_result = self.writing_file.recover(last_ckpt)?;
        result.append(&mut wf_result);

        Ok(result)
    }

    pub(crate) fn append_wal(
        &mut self,
        batch_op: BatchOp,
        last_ckpt: u64,
        fsync: bool,
    ) -> io::Result<()> {
        self.try_truncate_wal(last_ckpt);
        let bytes_to_append = batch_op.encode();
        let writing_file_len = self.writing_file.len()?;
        if bytes_to_append.len() as u64 + writing_file_len > self.max_size_per_file {
            self.switch_log_files();
        }
        self.writing_file
            .append_file(bytes_to_append, last_ckpt, fsync)
    }

    fn try_truncate_wal(&mut self, last_ckpt: u64) {
        if self.read_only_file.last_ckpt <= last_ckpt {
            // This place should spawn a thread to execute it.
            self.read_only_file.truncate();
        }
    }

    fn switch_log_files(&mut self) -> io::Result<usize> {
        self.writing_file.set_readonly_state()?;
        self.read_only_file.set_writing_state()?;
        std::mem::swap(&mut self.writing_file, &mut self.read_only_file);
        Ok(0)
    }
}

type Filestate = u8;

const READ_ONLY: Filestate = 0;
const WRITING: Filestate = 1;

const SIZE_OF_FILE_STATE: usize = 1; // Filestate type is u8.
const SIZE_OF_CKPT: usize = 8; // checkpoint type is u64.

struct LogFile {
    // The last last_ckpt in this log file.
    last_ckpt: u64,
    file: File,
}

impl LogFile {
    fn new(fpath: &'static str) -> (Self, Filestate) {
        let mut file = open_or_create_file(fpath);
        let file_len = file.metadata().unwrap().len();
        let mut state;
        if file_len == 0 {
            state = READ_ONLY;
            write_at(&mut file, &mut [READ_ONLY], 0)
                .expect(format!("init log_file {} with READONLY error", fpath).as_str());
        } else {
            let state_bytes = read_at(&mut file, 0, SIZE_OF_FILE_STATE)
                .expect(format!("read state from log_file {} error", fpath).as_str());
            state = bytes_to_u8(state_bytes.as_slice());
        }
        let mut last_ckpt;
        if file_len < SIZE_OF_CKPT as u64 {
            last_ckpt = 0;
        } else {
            let last_ckpt_bytes_offset = file_len - SIZE_OF_CKPT as u64;
            let last_ckpt_bytes =
                read_at(&file, last_ckpt_bytes_offset, SIZE_OF_CKPT).unwrap();
            last_ckpt = bytes_to_u64(last_ckpt_bytes.as_slice());
        }

        (Self { last_ckpt, file }, state)
    }

    fn append_file(&mut self, mut data: Vec<u8>, last_ckpt: u64, fsync: bool) -> io::Result<()> {
        data.append(&mut u64_to_bytes(last_ckpt));
        self.file.write_all(data.as_mut_slice())?;
        if fsync {
            self.file.sync_all()?;
        }
        self.last_ckpt = last_ckpt;
        Ok(())
    }

    fn recover(&mut self, last_ckpt: u64) -> io::Result<Vec<(Vec<KVpos>, Vec<u8>)>> {
        let result = Vec::new();
        let mut file_data = self.read_all()?.split_off(SIZE_OF_FILE_STATE);

        let (old_kvpos_len_bytes, left) = file_data.split_at(8);
        let (data_len_bytes, left2) = left.split_at(8);
        let old_kvpos_len = bytes_to_u64(old_kvpos_len_bytes);
        let data_len = bytes_to_u64(data_len_bytes);

        Ok(result)
    }

    fn read_all(&mut self) -> io::Result<Vec<u8>> {
        let mut data = Vec::new();
        self.file.read_to_end(&mut data)?;
        Ok(data)
    }

    fn truncate(&mut self) {
        self.file.set_len(SIZE_OF_FILE_STATE as u64);
        self.last_ckpt = 0;
    }

    fn get_last_ckpt(&self) -> u64 {
        self.last_ckpt
    }

    fn set_writing_state(&mut self) -> io::Result<usize> {
        write_at(&mut self.file, &mut [WRITING], 0)
    }

    fn set_readonly_state(&mut self) -> io::Result<usize> {
        write_at(&mut self.file, &mut [READ_ONLY], 0)
    }

    fn len(&self) -> io::Result<u64> {
        let metadata = self.file.metadata()?;
        Ok(metadata.len())
    }
}

pub(crate) struct AllOps(Vec<BatchOps>);

pub(crate) struct BatchOps{
    id: u64,
    ops: Vec<Ops>,
    undo: u64,
    checkpoint: u64,
}

impl BatchOps{
    pub(crate) fn encode(&self) -> Vec<u8>{
        let mut data = Vec::new();
        let mut iter = self.ops.iter();
        while let Some(ops) = iter.next() {
            let ops_bytes = ops.encode();
            data.append(bop_bytes);
        }
        data
    }

    pub(crate) fn decode(data: Vec<u8>) -> Self{

    }
}

type Operate = u8;
const INSERT: Operate = 0;
const DELETE: Operate = 1;

pub(crate) struct Ops{
    op: Operate,
    kv: KVpair,
}

impl Ops{
    pub(crate) fn encode(&self) -> Vec<u8>{

    }

    pub(crate) fn decode(data: Vec<u8>) -> Self{

    }
}

pub struct KVpair{
    key: Vec<u8>,
    value: Vec<u8>
}

/*
   fn to_bytes(multi_old_kvpos: Vec<KVpos>, mut data: Vec<u8>) -> Vec<u8> {
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
        bytes_to_append
    }
*/