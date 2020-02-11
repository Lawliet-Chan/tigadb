use std::fs::File;

pub(crate) struct Wal {
    wal_file: File,
}

impl Wal {
    pub fn new() -> Self {}
}
