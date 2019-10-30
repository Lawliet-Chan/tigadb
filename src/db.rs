use crate::option::Option;

pub struct DB {
    opt: Option,
}

impl DB {
    pub fn new(opt: Option) -> DB {
        DB { opt }
    }
}
