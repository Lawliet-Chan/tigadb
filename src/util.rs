use std::fs::File;
use std::path::Path;
use std::{io, u8};

pub(crate) fn open_or_create_file(fpath: &'static str) -> File {
    if Path::new(fpath).exists() {
        File::open(fpath).expect(format!("open file {} error", fpath).as_str())
    } else {
        File::create(fpath).expect(format!("create file {} error", fpath).as_str())
    }
}

pub(crate) fn read_at(file: &File, offset: u64, len: usize) -> io::Result<Vec<u8>> {
    let buf = &mut Vec::with_capacity(len);
    #[cfg(target_os = "unix")]
    {
        file.read_at(buf, offset)
    }

    #[cfg(target_os = "windows")]
    {
        file.seek_read(buf, offset)
    }
    Ok(buf.to_vec())
}

pub(crate) fn write_at(file: &mut File, buf: &mut [u8], offset: u64) -> io::Result<()> {
    #[cfg(target_os = "unix")]
    {
        file.write_at(buf, offset)
    }

    #[cfg(target_os = "windows")]
    {
        file.seek_write(buf, offset)
    }
    Ok(())
}

pub(crate) fn bytes_to_u8(data: Vec<u8>) -> u8 {
    let mut u8_1: [u8; 1] = [0_u8];
    u8_1.clone_from_slice(data.as_slice());
    u8::from_be_bytes(u8_1)
}
