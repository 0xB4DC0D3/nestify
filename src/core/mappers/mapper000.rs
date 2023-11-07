use crate::core::memory::Memory;

use super::Mapper;

pub struct Mapper000 {
    prg_rom: Vec<u8>,
    chr_rom: Vec<u8>,
    is_32kb_size: bool,
}

impl Mapper000 {
    pub fn new(prg_rom: Vec<u8>, chr_rom: Vec<u8>) -> Self {
        let is_32kb_size = prg_rom.len() > (16 * 1024);

        Self {
            prg_rom,
            chr_rom,
            is_32kb_size,
        }
    }
}

impl Memory for Mapper000 {
    fn read(&self, address: u16) -> u8 {
        match address {
            // Mapper 000 doesn't have RAM
            0x4020..=0x7FFF => 0x00,
            0x8000..=0xFFFF => {
                if !self.is_32kb_size {
                    self.prg_rom[(address as usize - 0x8000) & 0x3FFF]
                } else {
                    self.prg_rom[address as usize - 0x8000]
                }
            },
            _ => panic!("Invalid address for reading PRG-ROM!"),
        }
    }

    fn write(&mut self, _address: u16, _data: u8) {
        panic!("Attempt to write into PRG-ROM!");
    }
}

impl Mapper for Mapper000 {
    fn get_chr_rom(&mut self) -> &mut Vec<u8> {
        &mut self.chr_rom
    }
}
