mod screenbuffer;

use std::cell::RefCell;
use std::rc::Rc;

use self::screenbuffer::ScreenBuffer;

use super::bus::Bus;
use super::memory::Memory;
use super::registers::Register;
use super::registers::ppu::data::PpuDataRegister;
use super::registers::ppu::address::PpuAddressRegister;
use super::registers::ppu::scroll::PpuScrollRegister;
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
    scanline: isize,
    internal_buf: Option<u8>,
    screen_buffer: ScreenBuffer,
    trigger_zero_hit: bool,
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
            screen_buffer: ScreenBuffer::new(256, 240),
            trigger_zero_hit: false,
        }
    }

    pub fn tick(&mut self, amount: usize) {
        self.cycles += amount;

        let (scroll_x, scroll_y) = self.scroll.get_scroll();
        let x = (self.cycles - 2) as u16;
        let y = self.scanline as u16;

        let in_viewport_current_x = x >= scroll_x as u16;
        let in_viewport_current_y = y >= scroll_y as u16;
        let in_viewport_current = in_viewport_current_x && in_viewport_current_y;

        let show_background = self.mask.get_flag(PpuMaskRegisterFlags::ShowBackground);
        let show_sprites = self.mask.get_flag(PpuMaskRegisterFlags::ShowSprites);

        match self.cycles {
            1 => {
                match self.scanline {
                    241 => {
                        self.status.set_flag(PpuStatusRegisterFlags::VBlank, true);

                        if self.controller.get_flag(PpuControllerRegisterFlags::GenerateVBlankNMI) {
                            self.bus.borrow_mut().set_interrupt(Some(()));
                        }
                    },
                    -1 => {
                        self.status.set_flag(PpuStatusRegisterFlags::VBlank, false);
                        self.status.set_flag(PpuStatusRegisterFlags::SpriteZeroHit, false);
                        self.status.set_flag(PpuStatusRegisterFlags::SpriteOverflow, false);
                    },
                    _ => (),
                }
            },
            cycles @ (2..=257) if self.scanline >= 0 && self.scanline < 240 => {
                let x = (cycles - 2) as u16;
                let y = self.scanline as u16;
                let nametable_index = (self.controller.get() & 0b11) as u16;
                let bg_pattern_table = self.controller
                    .get_flag(PpuControllerRegisterFlags::BackgroundPatternTable);

                let pattern_table_index = if bg_pattern_table {
                    1u16
                } else {
                    0u16
                };

                let get_bg_color_index = |nametable_index| {
                    let tile_x = x / 8;
                    let tile_y = y / 8;
                    let tile_offset = tile_y * 32 + tile_x;

                    let tile_index = self.read(self.mirror_address(
                        0x2000 + 
                        nametable_index * 0x400 + 
                        tile_offset
                    )) as u16;

                    let lsb = self.read(self.mirror_address(
                        pattern_table_index * 0x1000 +
                        tile_index * 16 +
                        y % 8
                    ));

                    let msb = self.read(self.mirror_address(
                        pattern_table_index * 0x1000 +
                        tile_index * 16 +
                        y % 8 +
                        8
                    ));

                    let shift = 7 - x % 8;
                    let lsbit = (lsb >> shift) & 0b1;
                    let msbit = (msb >> shift) & 0b1;

                    ((msbit << 1) | lsbit) as u16
                };

                let get_palette_index = |nametable_index| {
                    let attribute_x = x / 32;
                    let attribute_y = y / 32;
                    let attribute_offset = attribute_y * 8 + attribute_x;

                    let attribute = self.read(self.mirror_address(
                        0x23C0 + 
                        nametable_index * 0x400 +
                        attribute_offset
                    ));

                    let attribute_area_x = (x % 32) / 16;
                    let attribute_area_y = (y % 32) / 16;

                    match (attribute_area_x, attribute_area_y) {
                        (0, 0) => (attribute >> 0) & 0b11,
                        (1, 0) => (attribute >> 2) & 0b11,
                        (0, 1) => (attribute >> 4) & 0b11,
                        (1, 1) => (attribute >> 6) & 0b11,
                        _ => panic!("Invalid attribute area!"),
                    }
                };

                let get_bg_color = |palette_index, color_index| {
                    if color_index == 0 {
                        self.read(0x3F00)
                    } else {
                        self.read(0x3F01 + palette_index as u16 * 0x4 + (color_index - 1))
                    }
                };

                let current_bg_color_index = get_bg_color_index(nametable_index);
                let current_bg_palette_index = get_palette_index(nametable_index);
                let current_bg_color = get_bg_color(current_bg_palette_index, current_bg_color_index);

                let nametable_offset = match self.mirroring {
                    Mirroring::Horizontal => 2,
                    Mirroring::Vertical => 1,
                    _ => todo!("Add four screen mirroring"),
                };

                let next_bg_color_index = get_bg_color_index((nametable_index + nametable_offset) & 0b11);
                let next_bg_palette_index = get_palette_index((nametable_index + nametable_offset) & 0b11);
                let next_bg_color = get_bg_color(next_bg_palette_index, next_bg_color_index);

                if show_background {
                    if in_viewport_current {
                        self.screen_buffer.set_pixel(
                            (x - scroll_x as u16) as usize,
                            (y - scroll_y as u16) as usize,
                            current_bg_color_index as u8,
                            current_bg_color
                        );
                    } else {
                        match (scroll_x > 0, scroll_y > 0) {
                            (true, false) => {
                                self.screen_buffer.set_pixel(
                                    (x + 256 - scroll_x as u16) as usize,
                                    y as usize,
                                    next_bg_color_index as u8,
                                    next_bg_color
                                );
                            },
                            (false, true) => {
                                self.screen_buffer.set_pixel(
                                    x as usize,
                                    (y + 240 - scroll_y as u16) as usize,
                                    next_bg_color_index as u8,
                                    next_bg_color
                                );
                            }
                            (true, true) => {
                                self.screen_buffer.set_pixel(
                                    (x + 256 - scroll_x as u16) as usize,
                                    (y + 240 - scroll_y as u16) as usize,
                                    next_bg_color_index as u8,
                                    next_bg_color
                                );
                            },
                            _ => (),
                        }
                    }
                }

                if show_sprites {
                    let (bg_color_index, _) = self.screen_buffer.get_pixel(x as usize, y as usize);
                    let sprite_pattern_table_index = if self.controller.get_flag(PpuControllerRegisterFlags::SpritesPatternTable) {
                        1u16
                    } else {
                        0u16
                    };

                    for (index, sprite) in self.get_oam().chunks(4).enumerate() {
                        let sprite_top_y = sprite[0] as u16;
                        let sprite_tile_index = sprite[1] as u16;
                        let sprite_palette = (sprite[2] & 0b11) as u16;
                        let priority = (sprite[2] >> 5) & 1 == 0;
                        let flip_horizontally = (sprite[2] >> 6) & 1 == 1;
                        let flip_vertically = (sprite[2] >> 7) & 1 == 1;
                        let sprite_top_x = sprite[3] as u16;

                        let hit_by_x = (sprite_top_x..sprite_top_x + 8).contains(&x);
                        let hit_by_y = (sprite_top_y..sprite_top_y + 8).contains(&(y + 1));

                        if !hit_by_x && !hit_by_y {
                            continue
                        }

                        let tile_y = if flip_vertically {
                            7 - y % 8
                        } else {
                            y % 8
                        };

                        let lsb = self.read(
                            sprite_pattern_table_index * 0x1000 +
                            sprite_tile_index * 16 + 
                            tile_y
                        );

                        let msb = self.read(
                            sprite_pattern_table_index * 0x1000 +
                            sprite_tile_index * 16 +
                            tile_y +
                            8
                        );

                        let shift = if !flip_horizontally {
                            7 - (x % 8)
                        } else {
                            x % 8
                        };

                        let lsbit = (lsb >> shift) & 0x1;
                        let msbit = (msb >> shift) & 0x1;
                        let spr_color_index = (msbit << 1) | lsbit;

                        let spr_color = self.read(0x3F10 + sprite_palette * 0x4 + spr_color_index as u16);

                        let coord_x = sprite_top_x + x % 8;
                        let coord_y = sprite_top_y + y % 8 + 1;

                        let draw_pixel = match (bg_color_index, spr_color_index) {
                            (0, 0) => false,
                            (0, 1..=3) => true,
                            (1..=3, 0) => false,
                            (1..=3, 1..=3) => {
                                if index == 0 && show_background {
                                    println!("scroll_top_x = {}, scroll_top_y = {}", sprite_top_x, sprite_top_y);
                                    self.trigger_zero_hit = true;
                                }

                                priority && show_background
                            }
                            _ => panic!("Invalid pixel!"),
                        };

                        if draw_pixel {
                            self.screen_buffer.set_pixel(
                                coord_x as usize,
                                coord_y as usize,
                                spr_color_index,
                                spr_color
                            );
                        }
                    }
                }

                if self.trigger_zero_hit {
                    self.status.set_flag(PpuStatusRegisterFlags::SpriteZeroHit, true);
                    self.trigger_zero_hit = false;
                }
            },
            341 => {
                self.cycles = 0;
                self.scanline += 1;

                if self.scanline >= 261 {
                    self.scanline = -1;
                }
            }
            _ => (),
        }
    }

    fn get_oam(&self) -> Vec<u8> {
        self.bus
            .borrow_mut()
            .ppu_memory_map()
            .get_oam()
            .to_vec()
    }

    pub fn get_screen_buffer(&self) -> &ScreenBuffer {
        &self.screen_buffer
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
