use std::{rc::Rc, cell::RefCell};

use super::ppu::Ppu;

pub struct Clock {
    ppu: Rc<RefCell<Ppu>>,
}

impl Clock {
    pub fn new(ppu: &Rc<RefCell<Ppu>>) -> Self {
        Self {
            ppu: ppu.clone(),
        }
    }

    pub fn tick(&self, amount: usize) {
        self.ppu.borrow_mut().tick(amount);
    }

    pub fn ppu(&self) -> &Rc<RefCell<Ppu>> {
        &self.ppu
    }
}
