use std::fs::File;
use std::os::unix::fs::FileExt;
use std::path::Path;
use std::{io, u16, u32, u8};

pub(crate) fn open_or_create_file(fpath: &'static str) -> File {
    if Path::new(fpath).exists() {
        File::open(fpath).expect(format!("open file {} error", fpath).as_str())
    } else {
        File::create(fpath).expect(format!("create file {} error", fpath).as_str())
    }
}

pub(crate) fn read_at(file: &File, offset: u64, len: usize) -> io::Result<Vec<u8>> {
    let buf = &mut Vec::with_capacity(len);
    file.read_at(buf, offset)?;
    Ok(buf.to_vec())
}

pub(crate) fn write_at(file: &mut File, buf: &mut [u8], offset: u64) -> io::Result<usize> {
    file.write_at(buf, offset)
}

pub(crate) fn bytes_to_u8(data: &[u8]) -> u8 {
    let mut u8_1: [u8; 1] = [0_u8];
    u8_1.clone_from_slice(data);
    u8::from_be_bytes(u8_1)
}

pub(crate) fn u8_to_bytes(u: u8) -> Vec<u8> {
    let u8_1: [u8; 1] = u.to_be_bytes();
    u8_1.to_vec()
}

pub(crate) fn bytes_to_u16(data: &[u8]) -> u16 {
    let mut u8_2: [u8; 2] = [0_u8; 2];
    u8_2.clone_from_slice(data);
    u16::from_be_bytes(u8_2)
}

pub(crate) fn u16_to_bytes(u: u16) -> Vec<u8> {
    let u8_2: [u8; 2] = u.to_be_bytes();
    u8_2.to_vec()
}

pub(crate) fn bytes_to_u32(data: &[u8]) -> u32 {
    let mut u8_4: [u8; 4] = [0_u8; 4];
    u8_4.clone_from_slice(data);
    u32::from_be_bytes(u8_4)
}

pub(crate) fn u32_to_bytes(u: u32) -> Vec<u8> {
    let u8_4: [u8; 4] = u.to_be_bytes();
    u8_4.to_vec()
}
