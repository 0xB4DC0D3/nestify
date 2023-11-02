use crate::core::memory::Memory;

use super::Mapper;

pub struct Mapper000 {
    prg_rom: Vec<u8>,
    chr_rom: Vec<u8>,
}

impl Mapper000 {
    pub fn new(prg_rom: Vec<u8>, chr_rom: Vec<u8>) -> Self {
        Self {
            prg_rom,
            chr_rom,
        }
    }
}

impl Memory for Mapper000 {
    fn read(&self, address: u16) -> u8 {
        match address {
            0x8000..=0xFFFF => self.prg_rom[address as usize - 0x8000],
            _ => panic!("Invalid address for reading PRG-ROM!"),
        }
    }

    fn write(&mut self, _address: u16, _data: u8) {
        panic!("Attempt to write into PRG-ROM!");
    }
}

impl Mapper for Mapper000 {
    fn get_chr_rom(&self) -> &Vec<u8> {
        &self.chr_rom
    }
}
