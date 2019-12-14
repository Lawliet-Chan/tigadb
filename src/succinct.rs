pub struct Succinct {}

impl Succinct {
    pub fn new() -> Self {}

    pub fn rank0(&self, x: u64) -> u64 {}

    pub fn select0(&self, x: u64) -> u64 {}

    pub fn rank1(&self, x: u64) -> u64 {}

    pub fn select1(&self, x: u64) -> u64 {}

    pub fn first_child(&self, i: u64) -> u64 {
        self.select0(self.rank1(i)) + 1
    }

    pub fn last_child(&self, i: u64) -> u64 {
        self.select0(self.rank1(i) + 1) - 1
    }

    pub fn parent(&self, i: u64) -> u64 {
        self.select1(self.rank0(i))
    }

    pub fn children(&self, i: u64) -> u64 {
        self.last_child(i) - self.first_child(i) + 1
    }

    pub fn child(&self, i: u64, num: u64) -> u64 {
        self.first_child(i) + num
    }
}
