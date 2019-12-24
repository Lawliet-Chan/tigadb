pub struct Option {
    fsync: bool,
    limit_per_file: usize,
}

impl Default for Option {
    fn default() -> Self {
        Self {
            fsync: true,
            limit_per_file: 2 * 1024 * 1024 * 1024,
        }
    }
}
