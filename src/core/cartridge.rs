use std::cell::RefCell;
use std::rc::Rc;

use super::ppu::Mirroring;
use super::mappers::Mapper;
use super::mappers::Mapper000;

pub struct Cartridge {
    mirroring: Mirroring,
    mapper: Rc<RefCell<Box<dyn Mapper>>>,
}

impl Cartridge {
    pub fn new(rom: Vec<u8>) -> Self {
        rom.get(0..16).expect("Unable to parse NES Header, possibly wrong file!");

        if &rom[0..4] != b"NES\x1A" {
            panic!("This ROM is not iNES format!");
        }

        let prg_rom_size = rom[4] as u16;
        let chr_rom_size = rom[5] as u16;

        let flag6_metadata = rom[6];
        let (mirroring, mapper_lower_nybble, has_trainer, has_batterybacked_prg_ram) = {
            let four_screen_mirroring = (flag6_metadata >> 3) & 0x1 == 0x1;
            let mirroring = if four_screen_mirroring {
                Mirroring::FourScreen
            } else {
                match flag6_metadata & 0x1 {
                    0x00 => Mirroring::Horizontal,
                    0x01 => Mirroring::Vertical,
                    _ => panic!("Invalid Flag6, could not happen!"),
                }
            };

            let has_batterybacked_prg_ram = (flag6_metadata >> 1) & 0x1 == 0x1;
            let has_trainer = (flag6_metadata >> 2) & 0x1 == 0x1;
            let mapper_lower_nybble = flag6_metadata >> 4;

            (mirroring, mapper_lower_nybble, has_trainer, has_batterybacked_prg_ram)
        };

        // If it's iNES 2.0 format, flags 8-15 are in NES 2.0 format
        let flag7_metadata = rom[7];
        let (mapper_upper_nybble, is_nes20_format, is_playchoice10, is_vsunisystem) = {
            let mapper_upper_nybble = flag7_metadata >> 4;
            let is_nes20_format = (flag7_metadata >> 2) & 0x3 == 0x2;
            let is_playchoice10 = (flag7_metadata >> 1) & 0x1 == 0x1;
            let is_vsunisystem = flag7_metadata & 0x1 == 0x1;

            (mapper_lower_nybble, is_nes20_format, is_playchoice10, is_vsunisystem)
        };

        let prg_ram_size = rom[8];

        let (prg_rom_begin, prg_rom_end) = {
            let begin = if has_trainer { 16 + 512 } else { 16 };
            let end = begin + prg_rom_size * 16 * 1024;

            (begin as usize, end as usize)
        };

        let (chr_rom_begin, chr_rom_end) = {
            let begin = prg_rom_end;
            let end = begin + chr_rom_size as usize * 8 * 1024;

            (begin, end)
        };

        let prg_rom = rom
            .get(prg_rom_begin..prg_rom_end)
            .clone()
            .expect("Unable to get PRG-ROM!")
            .to_vec();

        let chr_rom = rom
            .get(chr_rom_begin..chr_rom_end)
            .clone()
            .expect("Unable to get CHR-ROM!")
            .to_vec();

        // TODO: add more mappers later
        let mapper: Box<dyn Mapper> = match mapper_lower_nybble {
            0 => Box::new(Mapper000::new(prg_rom, chr_rom)),
            _ => panic!("Unsupported mapper!"),
        };
        
        Self {
            mirroring,
            mapper: Rc::new(RefCell::new(mapper)),
        }
    }

    pub fn get_mirroring(&self) -> Mirroring {
        self.mirroring
    }

    pub fn empty() -> Self {
        let mapper = Box::new(Mapper000::new(vec![0; 0x8000], vec![0; 0x2000]));

        Self {
            mirroring: Mirroring::Horizontal,
            mapper: Rc::new(RefCell::new(mapper)),
        }
    }

    pub fn get_mapper(&self) -> &Rc<RefCell<Box<dyn Mapper>>> {
        &self.mapper
    }
}
