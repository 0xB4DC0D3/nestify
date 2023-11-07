use std::cell::RefCell;
use std::rc::Rc;

use super::bus::Bus;
use super::memory::Memory;
use super::registers::Register;
use super::registers::ppu::data::PpuDataRegister;
use super::registers::ppu::address::{PpuAddressRegister, self};
use super::registers::ppu::scroll::{PpuScrollRegister, PpuScrollRegisterFlags};
use super::registers::ppu::oamdata::PpuOamDataRegister;
use super::registers::ppu::oamaddress::PpuOamAddressRegister;
use super::registers::ppu::status::{PpuStatusRegister, PpuStatusRegisterFlags};
use super::registers::ppu::mask::{PpuMaskRegister, PpuMaskRegisterFlags};
use super::registers::ppu::controller::{PpuControllerRegister, PpuControllerRegisterFlags};

#[repr(u8)]
#[derive(Clone, Copy)]
pub enum Mirroring {
    Horizontal,
    Vertical,
    FourScreen,
}

pub struct Ppu {
    mirroring: Mirroring,
    controller: PpuControllerRegister,
    mask: PpuMaskRegister,
    status: PpuStatusRegister,
    oamaddress: PpuOamAddressRegister,
    oamdata: PpuOamDataRegister,
    scroll: PpuScrollRegister,
    address: PpuAddressRegister,
    data: PpuDataRegister,
    bus: Rc<RefCell<Bus>>,
    cycles: usize,
    scanline: usize,
    internal_buf: Option<u8>,
}

impl Ppu {
    pub fn new(bus: &Rc<RefCell<Bus>>, mirroring: Mirroring) -> Self {
        Self {
            mirroring,
            controller: PpuControllerRegister::new(),
            mask: PpuMaskRegister::new(),
            status: PpuStatusRegister::new(),
            oamaddress: PpuOamAddressRegister::new(),
            oamdata: PpuOamDataRegister::new(),
            scroll: PpuScrollRegister::new(),
            address: PpuAddressRegister::new(),
            data: PpuDataRegister::new(),
            bus: bus.clone(),
            cycles: 0, 
            scanline: 0,
            internal_buf: None,
        }
    }

    pub fn is_sprite_zero_hit(&self, cycles: usize) -> bool {
        let show_sprites = self.mask.get_flag(PpuMaskRegisterFlags::ShowSprites);
        let y = self.bus
            .borrow_mut()
            .ppu_memory_map()
            .get_oam()
            .get(0)
            .cloned()
            .expect("Unable to get Y coordinate!") as usize;

        let x = self.bus
            .borrow_mut()
            .ppu_memory_map()
            .get_oam()
            .get(3)
            .cloned()
            .expect("Unable to get X coordinate!") as usize;

        (y == self.scanline) && x <= cycles && show_sprites
    }

    pub fn tick(&mut self, amount: usize) {
        self.cycles += amount;

        if self.cycles >= 341 {
            if self.is_sprite_zero_hit(self.cycles) {
                self.status.set_flag(PpuStatusRegisterFlags::SpriteZeroHit, true);
            }

            self.cycles -= 341;
            self.scanline += 1;

            if self.scanline == 241 {
                self.status.set_flag(PpuStatusRegisterFlags::VBlank, true);
                self.status.set_flag(PpuStatusRegisterFlags::SpriteZeroHit, false);

                if self.controller.get_flag(PpuControllerRegisterFlags::GenerateVBlankNMI) {
                    self.bus.borrow_mut().set_interrupt(Some(()));
                }
            }

            if self.scanline >= 262 {
                self.scanline = 0;
                self.bus.borrow_mut().set_interrupt(None);
                self.status.set_flag(PpuStatusRegisterFlags::SpriteZeroHit, false);
                self.status.set_flag(PpuStatusRegisterFlags::VBlank, false);
            }
        }
    }

    fn read_range(&self, start: u16, amount: u16) -> Vec<u8> {
        (start..start + amount)
            .into_iter()
            .map(|address| {
                self.read(self.mirror_address(address))
            })
            .collect::<Vec<_>>()
    }

    pub fn get_nametable(&self, nametable_index: u16) -> Vec<u8> {
        self.read_range(0x2000 + 0x400 * nametable_index, 0x400)
    }

    pub fn get_pattern_table(&self, pattern_table_index: u16) -> Vec<u8> {
        let chr_rom_buf = self.read_range(0x1000 * pattern_table_index, 0x1000);

        chr_rom_buf
            .chunks(16)
            .flat_map(|chunk| {
                chunk
                    .iter()
                    .zip(chunk.iter().skip(8))
                    .flat_map(|(&lsbyte, &msbyte)| {
                        (0..8)
                            .rev()
                            .map(|shift| {
                                let lsbit = (lsbyte >> shift) & 1;
                                let msbit = (msbyte >> shift) & 1;
                                (msbit << 1) | lsbit
                            })
                            .collect::<Vec<_>>()
                    })
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>()
    }

    pub fn get_palette_table(&self, palette_index: u16) -> Vec<u8> {
        let palette = self.read_range(0x3F00 + palette_index * 0x4 + 1, 0x4);

        vec![palette[0], palette[1], palette[2], self.read(0x3F00)]
    }

    pub fn get_sprite_palette_table(&self, palette_index: u16) -> Vec<u8> {
        self.read_range(0x3F10 + palette_index * 0x4 + 1, 0x4)
    }

    pub fn get_attribute_table(&self, nametable_index: u16) -> Vec<u8> {
        self.read_range(0x2000 + nametable_index * 0x400 + 0x3C0, 0x40)
    }

    pub fn get_nametable_index(&self) -> u16 {
        (self.controller.get() & 0b11) as u16
    }

    pub fn get_bg_pattern_table_index(&self) -> u16 {
        self.controller.get_flag(PpuControllerRegisterFlags::BackgroundPatternTable) as u16
    }

    pub fn get_spr_patter_table_index(&self) -> u16 {
        self.controller.get_flag(PpuControllerRegisterFlags::SpritesPatternTable) as u16
    }

    pub fn get_second_nametable_index(&self, nametable_index: u16) -> u16 {
        match self.mirroring {
            Mirroring::Vertical => {
                match nametable_index {
                    0 => 1,
                    1 => 0,
                    _ => panic!("Invalid nametable index!"),
                }
            },
            Mirroring::Horizontal => {
                match nametable_index {
                    0 => 2,
                    2 => 0,
                    _ => panic!("Invalid nametable index!"),
                }
            },
            _ => panic!("Currently unsupported mirroring!"),
        }
    }

    pub fn get_background_buffer(&self) -> (Vec<u8>, Vec<u8>) {
        let nametable_index = self.get_nametable_index();
        let second_nametable_index = self.get_second_nametable_index(nametable_index);
        let pattern_table_index = self.get_bg_pattern_table_index();

        let pattern_table = self.get_pattern_table(pattern_table_index);
        let create_buffer = |nametable_index: u16| {
            self.get_nametable(nametable_index)
                .iter()
                .take(0x3C0)
                .flat_map(|&pattern_table_index| {
                    let index = pattern_table_index as usize * 64;

                    pattern_table
                        .get(index..index + 64)
                        .expect("Unable to get data from Pattern table!")
                        .to_vec()
                })
                .collect()
        };

        let main_buffer = create_buffer(nametable_index);
        let second_buffer = create_buffer(second_nametable_index);

        (main_buffer, second_buffer)
    }

    pub fn get_scroll(&self) -> (u8, u8) {
        self.scroll.get_scroll()
    }

    pub fn get_sprites_buffer(&self) -> Vec<u8> {
        self.bus.borrow_mut().ppu_memory_map().get_oam().to_vec()
    }

    pub fn has_interrupt(&self) -> bool {
        self.bus.borrow().get_interrupt().is_some()
    }

    pub fn mirror_address(&self, address: u16) -> u16 {
        let nametable_index = (address - 0x2000) / 0x400;
        match (self.mirroring, nametable_index) {
            (Mirroring::Horizontal, 1) | (Mirroring::Horizontal, 3) => address - 0x400,
            (Mirroring::Vertical, 2) | (Mirroring::Vertical, 3) => address - 0x800,
            (Mirroring::FourScreen, _) => todo!("Add Four screen mirroring!"),
            _ => address,
        }
    }

    pub fn write_controller(&mut self, data: u8) {
        let already_in_vblank = self.controller.get_flag(PpuControllerRegisterFlags::GenerateVBlankNMI);
        let status_vblank = self.status.get_flag(PpuStatusRegisterFlags::VBlank);

        self.controller.set(data);
        let in_vblank = self.controller.get_flag(PpuControllerRegisterFlags::GenerateVBlankNMI);

        if !already_in_vblank && status_vblank && in_vblank {
            self.bus.borrow_mut().set_interrupt(Some(()));
        }
    }

    pub fn write_mask(&mut self, data: u8) {
        self.mask.set(data);
    }

    pub fn write_oamaddress(&mut self, data: u8) {
        self.oamaddress.set(data);
    }

    pub fn write_oamdata(&mut self, data: u8) {
        let oamaddress = self.oamaddress.get();

        self.bus
            .borrow_mut()
            .ppu_memory_map()
            .set_oam_value(oamaddress, data);

        self.oamaddress.set(oamaddress.wrapping_add(1));
        self.oamdata.set(data);
    }

    pub fn write_scroll(&mut self, data: u8) {
        self.scroll.set(data);
    }

    pub fn write_address(&mut self, data: u8) {
        self.address.set(data);
    }

    pub fn write_data(&mut self, data: u8) {
        let address_increment = self.controller.get_flag(PpuControllerRegisterFlags::AddressIncrement);
        let address = self.address.get_address();

        let write_address = match address {
            0x2000..=0x2FFF => self.mirror_address(address),
            0x3000..=0x3EFF => self.mirror_address(address - 0x1000),
            _ => address,
        };

        self.write(write_address, data);
        self.data.set(data);
        self.address.set_address(if address_increment {
            address.wrapping_add(32)
        } else {
            address.wrapping_add(1)
        });
    }

    pub fn write_oamdma(&mut self, data: u8) {
        let start = u16::from_le_bytes([0x00, data]);
        let end = start + 0x100;
        let oam_buf = (start..end)
            .into_iter()
            .map(|address| {
                self.bus
                    .borrow_mut()
                    .cpu_memory_map()
                    .read(address)
            })
            .collect::<Vec<_>>();

        self.bus
            .borrow_mut()
            .ppu_memory_map()
            .set_oam_buf(&oam_buf);
    }

    pub fn read_status(&mut self) -> u8 {
        let result = self.status.get();

        self.status.set_flag(PpuStatusRegisterFlags::VBlank, false);
        self.scroll.reset_latch();
        self.address.reset_latch();

        result
    }

    pub fn read_oamdata(&mut self) -> u8 {
        *self.bus
            .borrow_mut()
            .ppu_memory_map()
            .get_oam()
            .get(self.oamaddress.get() as usize)
            .expect("Unable to read from OAM!")
    }

    pub fn read_data(&mut self) -> u8 {
        let internal_buf = self.internal_buf.unwrap_or(0);
        let address_increment = self.controller.get_flag(PpuControllerRegisterFlags::AddressIncrement);
        let address = self.address.get_address();
        let read_address = match address {
            0x2000..=0x2FFF => self.mirror_address(address),
            0x3000..=0x3EFF => self.mirror_address(address - 0x1000),
            _ => address,
        };

        self.address.set_address(if address_increment {
            address.wrapping_add(32)
        } else {
            address.wrapping_add(1)
        });

        match address {
            0x0000..=0x3EFF => {
                self.internal_buf = Some(self.read(read_address));

                internal_buf
            }
            _ => self.read(read_address),
        }
    }
}

impl Memory for Ppu {
    fn read(&self, address: u16) -> u8 {
        self.bus
            .borrow_mut()
            .ppu_memory_map()
            .read(address)
    }

    fn write(&mut self, address: u16, data: u8) {
        self.bus
            .borrow_mut()
            .ppu_memory_map()
            .write(address, data);
    }
}
