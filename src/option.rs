pub struct Option {
    fsync: bool,
}

impl Default for Option {
    fn default() -> Self {
        Self { fsync: true }
    }
}
