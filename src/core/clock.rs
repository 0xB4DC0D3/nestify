use std::rc::Rc;
use std::cell::RefCell;

use super::ppu::Ppu;

pub struct Clock {
    ppu: Rc<RefCell<Ppu>>,
    render_callback: Box<dyn FnMut(&Ppu)>,
    cycles: usize,
}

impl Clock {
    pub fn new<F>(ppu: &Rc<RefCell<Ppu>>, render_callback: F) -> Self
    where F: FnMut(&Ppu) + 'static
    {
        Self {
            ppu: ppu.clone(),
            render_callback: Box::new(render_callback),
            cycles: 7,
        }
    }

    pub fn reset(&mut self) {
        self.cycles = 7;
    }

    pub fn tick(&mut self, amount: usize) {
        self.cycles += amount;
        let nmi_interrupt_before = self.ppu.borrow().has_interrupt();

        for _ in 0..(amount * 3) {
            self.ppu.borrow_mut().tick(1);
        }

        let nmi_interrupt_after = self.ppu.borrow().has_interrupt();

        if !nmi_interrupt_before && nmi_interrupt_after {
            (*self.render_callback)(&*self.ppu.borrow());
        }
    }

    pub fn get_cycles(&self) -> usize {
        self.cycles
    }

    pub fn ppu(&self) -> &Rc<RefCell<Ppu>> {
        &self.ppu
    }
}
