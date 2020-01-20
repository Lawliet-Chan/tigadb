#[derive(Copy, Clone)]
pub struct Option {
    pub fsync: bool,
    pub limit_per_file: u64,
    pub meta_dir: &'static str,
    pub kv_dir: &'static str,
}

impl Default for Option {
    fn default() -> Self {
        Self {
            fsync: true,
            limit_per_file: 2 * 1024 * 1024 * 1024,
            meta_dir: "tigadb/meta",
            kv_dir: "tigadb/kv",
        }
    }
}
