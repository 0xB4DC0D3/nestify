// TODO: add total_cycles for disassembling info in the future
pub struct Clock {
    cycles: usize,
}

impl Clock {
    pub fn new() -> Self {
        Self {
            cycles: 0,
        }
    }

    pub fn tick(&mut self, amount: usize) {
        self.cycles = amount;
    }
}
