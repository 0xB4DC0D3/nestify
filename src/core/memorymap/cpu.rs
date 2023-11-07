use std::rc::Rc;
use std::cell::RefCell;

use crate::core::mappers::Mapper;
use crate::core::memory::Memory;

use super::MemoryMap;

pub struct CpuMemoryMap {
    internal_ram: [u8; 0x800],
    mapper: Rc<RefCell<Box<dyn Mapper>>>,
}

impl CpuMemoryMap {
    pub fn new(mapper: &Rc<RefCell<Box<dyn Mapper>>>) -> Self {
        Self {
            internal_ram: [0; 0x800],
            mapper: mapper.clone(),
        }
    }
}

impl Memory for CpuMemoryMap {
    fn read(&self, address: u16) -> u8 {
        match address {
            0x0000..=0x1FFF => self.internal_ram[address as usize & 0x7FF],
            0x4020..=0xFFFF => self.mapper.borrow_mut().read(address),
            _ => panic!("Unable to read from address {:#04X} in CPU Memory Map!", address),
        }
    }

    fn write(&mut self, address: u16, data: u8) {
        match address {
            0x0000..=0x1FFF => {
                self.internal_ram[address as usize & 0x7FF] = data;
            },
            0x8000..=0xFFFF => panic!("Attempt to write into PRG-ROM in CPU Memory Map!"),
            _ => (),
        }
    }
}

impl MemoryMap for CpuMemoryMap {}
