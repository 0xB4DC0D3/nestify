use std::ops::IndexMut;
use std::rc::Rc;
use std::cell::RefCell;

use crate::core::mappers::Mapper;
use crate::core::memory::Memory;

use super::MemoryMap;

pub struct PpuMemoryMap {
    nametable: [u8; 0x1000],
    palette: [u8; 0x20],
    oam: [u8; 0x100],
    mapper: Rc<RefCell<Box<dyn Mapper>>>,
}

impl PpuMemoryMap {
    pub fn new(mapper: &Rc<RefCell<Box<dyn Mapper>>>) -> Self {
        Self {
            nametable: [0; 0x1000],
            palette: [0; 0x20],
            oam: [0; 0x100],
            mapper: mapper.clone(),
        }
    }

    pub fn get_oam(&self) -> &[u8; 0x100] {
        &self.oam
    }

    pub fn set_oam_value(&mut self, address: u8, value: u8) {
        self.oam[address as usize] = value;
    }

    pub fn set_oam_buf(&mut self, buf: &Vec<u8>) {
        self.oam.copy_from_slice(buf);
    }
}

impl Memory for PpuMemoryMap {
    fn read(&self, address: u16) -> u8 {
        match address {
            0x0000..=0x1FFF => {
                self.mapper
                    .borrow_mut()
                    .get_chr_rom()
                    .get(address as usize)
                    .cloned()
                    .expect("Unable to get value from Pattern table!")
            },
            0x2000..=0x2FFF => self.nametable[address as usize - 0x2000],
            0x3000..=0x3EFF => self.nametable[address as usize - 0x3000],
            0x3F10 | 0x3F14 | 0x3F18 | 0x3F1C => self.palette[address as usize - 0x3F10],
            0x3F00..=0x3FFF => self.palette[address as usize & 0x3F1F - 0x3F00],
            _ => panic!("Unable to read from address {:#04X} in CPU Memory Map!", address),
        }
    }

    fn write(&mut self, address: u16, data: u8) {
        match address {
            0x0000..=0x1FFF => {
                let mut mapper = self.mapper.borrow_mut();
                let chr_rom = mapper.get_chr_rom();
                let pattern_table_cell = chr_rom.index_mut(address as usize);

                *pattern_table_cell = data;
            },
            0x2000..=0x2FFF => {
                self.nametable[address as usize - 0x2000] = data;
            },
            0x3000..=0x3EFF => {
                self.nametable[address as usize - 0x3000] = data;
            },
            0x3F10 | 0x3F14 | 0x3F18 | 0x3F1C => {
                self.palette[address as usize - 0x3F10] = data;
            },
            0x3F00..=0x3FFF => {
                self.palette[address as usize & 0x3F1F - 0x3F00] = data;
            },
            _ => panic!("Unable to read from address {:#04X} in CPU Memory Map!", address),
        }
    }
}

impl MemoryMap for PpuMemoryMap {}
