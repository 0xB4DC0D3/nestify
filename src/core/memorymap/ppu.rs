use crate::core::memory::Memory;

use super::MemoryMap;
use super::MemoryMapType;

pub struct PpuMemoryMap {
    pattern_table: [u8; 0x2000],
    nametable: [u8; 0x1000],
    palette: [u8; 0x20],
}

impl PpuMemoryMap {
    pub fn new() -> Self {
        Self {
            pattern_table: [0; 0x2000],
            nametable: [0; 0x1000],
            palette: [0; 0x20],
        }
    }
}

impl Memory for PpuMemoryMap {
    fn read(&self, address: u16) -> u8 {
        match address {
            0x0000..=0x1FFF => self.pattern_table[address as usize],
            0x2000..=0x2FFF => self.nametable[address as usize - 0x2000],
            0x3000..=0x3EFF => self.nametable[address as usize - 0x3000],
            0x3F10 | 0x3F14 | 0x3F18 | 0x3F1C => self.nametable[address as usize - 0x3F10],
            0x3F00..=0x3F1F => self.palette[address as usize - 0x3F00],
            0x3F20..=0x3FFF => self.palette[address as usize & 0x3F1F - 0x3F00],
            _ => panic!("Unable to read from address {:#04X} in CPU Memory Map!", address),
        }
    }

    fn write(&mut self, address: u16, data: u8) {
        match address {
            0x0000..=0x1FFF => {
                self.pattern_table[address as usize] = data;
            },
            0x2000..=0x2FFF => {
                self.nametable[address as usize - 0x2000] = data;
            },
            0x3000..=0x3EFF => {
                self.nametable[address as usize - 0x3000] = data;
            },
            0x3F10 | 0x3F14 | 0x3F18 | 0x3F1C => {
                self.nametable[address as usize - 0x3F10] = data;
            },
            0x3F00..=0x3F1F => {
                self.palette[address as usize - 0x3F00] = data;
            },
            0x3F20..=0x3FFF => {
                self.palette[address as usize & 0x3F1F - 0x3F00] = data;
            },
            _ => panic!("Unable to read from address {:#04X} in CPU Memory Map!", address),
        }
    }
}

impl MemoryMap for PpuMemoryMap {
    fn get_type(&self) -> MemoryMapType {
        MemoryMapType::Ppu
    }
}
