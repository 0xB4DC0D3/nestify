mod screenbuffer;
mod screenstate;

use std::cell::RefCell;
use std::rc::Rc;

use self::screenbuffer::ScreenBuffer;
use self::screenstate::ScreenState;

use super::bus::Bus;
use super::memory::Memory;
use super::registers::Register;
use super::registers::ppu::data::PpuDataRegister;
use super::registers::ppu::oamdata::PpuOamDataRegister;
use super::registers::ppu::oamaddress::PpuOamAddressRegister;
use super::registers::ppu::status::{PpuStatusRegister, PpuStatusRegisterFlags};
use super::registers::ppu::mask::{PpuMaskRegister, PpuMaskRegisterFlags};
use super::registers::ppu::controller::{PpuControllerRegister, PpuControllerRegisterFlags};
use super::registers::ppu::vram::PpuVRamRegister;

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
    vram: PpuVRamRegister,
    vram_temp: PpuVRamRegister,
    data: PpuDataRegister,
    bus: Rc<RefCell<Bus>>,
    address_latch: bool,
    fine_x: u16,
    cycles: usize,
    scanline: isize,
    internal_buf: Option<u8>,
    screen_state: ScreenState,
    screen_buffer: ScreenBuffer,
    internal_oam: [u8; 0x20],
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
            vram: PpuVRamRegister::new(),
            vram_temp: PpuVRamRegister::new(),
            data: PpuDataRegister::new(),
            bus: bus.clone(),
            address_latch: false,
            fine_x: 0,
            cycles: 0, 
            scanline: 0,
            internal_buf: None,
            screen_state: ScreenState::new(),
            screen_buffer: ScreenBuffer::new(256, 240),
            internal_oam: [0xFF; 0x20],
        }
    }

    fn increment_scroll_x(&mut self) {
        let show_background = self.mask.get_flag(PpuMaskRegisterFlags::ShowBackground);
        let show_sprites = self.mask.get_flag(PpuMaskRegisterFlags::ShowSprites);

        if show_background || show_sprites {
            let coarse_x = self.vram.get_coarse_x();

            match coarse_x {
                31 => {
                    let nametable_x = self.vram.get_nametable_x();

                    self.vram.set_coarse_x(0);
                    self.vram.set_nametable_x(!nametable_x);
                },
                _ => {
                    self.vram.set_coarse_x(coarse_x + 1);
                },
            }
        }
    }

    fn increment_scroll_y(&mut self) {
        let fine_y = self.vram.get_fine_y();
        let show_background = self.mask.get_flag(PpuMaskRegisterFlags::ShowBackground);
        let show_sprites = self.mask.get_flag(PpuMaskRegisterFlags::ShowSprites);

        if show_background || show_sprites {
            if fine_y < 7 {
                self.vram.set_fine_y(fine_y + 1);
            } else {
                self.vram.set_fine_y(0);

                let coarse_y = self.vram.get_coarse_y();

                match coarse_y {
                    29 => {
                        let nametable_y = self.vram.get_nametable_y();

                        self.vram.set_coarse_y(0);
                        self.vram.set_nametable_y(!nametable_y);
                    },
                    31 => {
                        self.vram.set_coarse_y(0);
                    },
                    _ => {
                        self.vram.set_coarse_y(coarse_y + 1);
                    }
                }
            }
        }
    }

    fn transfer_address_x(&mut self) {
        let show_background = self.mask.get_flag(PpuMaskRegisterFlags::ShowBackground);
        let show_sprites = self.mask.get_flag(PpuMaskRegisterFlags::ShowSprites);

        if show_background || show_sprites {
            self.vram.set_nametable_x(self.vram_temp.get_nametable_x());
            self.vram.set_coarse_x(self.vram_temp.get_coarse_x());
        }
    }

    fn transfer_address_y(&mut self) {
        let show_background = self.mask.get_flag(PpuMaskRegisterFlags::ShowBackground);
        let show_sprites = self.mask.get_flag(PpuMaskRegisterFlags::ShowSprites);

        if show_background || show_sprites {
            self.vram.set_fine_y(self.vram_temp.get_fine_y());
            self.vram.set_nametable_y(self.vram_temp.get_nametable_y());
            self.vram.set_coarse_y(self.vram_temp.get_coarse_y());
        }
    }

    fn read_tile_id(&mut self) {
        let vram_address = self.vram.get();
        let tile_id = self.read(0x2000 | (vram_address & 0x0FFF));
        self.screen_state.bg_next_tile_id = tile_id;
    }

    fn read_attribute(&mut self) {
        let mut attribute = self.read(
            0x23C0 |
            (self.vram.get_nametable_y() << 11) |
            (self.vram.get_nametable_x() << 10) |
            ((self.vram.get_coarse_y() >> 2) << 3) |
            (self.vram.get_coarse_x() >> 2)
        );

        if self.vram.get_coarse_y() & 0b10 != 0 {
            attribute >>= 4;
        }

        if self.vram.get_coarse_x() & 0b10 != 0 {
            attribute >>= 2;
        }

        self.screen_state.bg_next_tile_attribute = attribute & 0b11;
    }

    fn read_tile_lsb(&mut self) {
        let bg_pattern_table = if self.controller.get_flag(PpuControllerRegisterFlags::BackgroundPatternTable) {
            1u16
        } else {
            0u16
        };

        let tile_lsb = self.read(
            (bg_pattern_table << 12) +
            ((self.screen_state.bg_next_tile_id as u16) << 4) +
            self.vram.get_fine_y()
        );

        self.screen_state.bg_next_tile_lsb = tile_lsb;
    }

    fn read_tile_msb(&mut self) {
        let bg_pattern_table = if self.controller.get_flag(PpuControllerRegisterFlags::BackgroundPatternTable) {
            1u16
        } else {
            0u16
        };

        let tile_msb = self.read(
            (bg_pattern_table << 12) +
            ((self.screen_state.bg_next_tile_id as u16) << 4) +
            self.vram.get_fine_y() +
            8
        );

        self.screen_state.bg_next_tile_msb = tile_msb;
    }

    fn load_background_shift(&mut self) {
        let tile_lsb = self.screen_state.bg_next_tile_lsb;
        let tile_msb = self.screen_state.bg_next_tile_msb;
        let attribute = self.screen_state.bg_next_tile_attribute;

        let shift_pattern_lo = self.screen_state.bg_shift_pattern_lo;
        let shift_pattern_hi = self.screen_state.bg_shift_pattern_hi;

        let shift_attribute_lo = self.screen_state.bg_shift_attribute_lo;
        let shift_attribute_hi = self.screen_state.bg_shift_attribute_hi;

        self.screen_state.bg_shift_pattern_lo = (shift_pattern_lo & 0xFF00) | tile_lsb as u16;
        self.screen_state.bg_shift_pattern_hi = (shift_pattern_hi & 0xFF00) | tile_msb as u16;

        self.screen_state.bg_shift_attribute_lo = 
            (shift_attribute_lo & 0xFF00) | if attribute & 0b01 != 0 {
                0xFF
            } else {
                0x00
            };

        self.screen_state.bg_shift_attribute_hi = 
            (shift_attribute_hi & 0xFF00) | if attribute & 0b10 != 0 {
                0xFF
            } else {
                0x00
            };
    }

    fn update_shift(&mut self) {
        let show_background = self.mask.get_flag(PpuMaskRegisterFlags::ShowBackground);
        let show_sprites = self.mask.get_flag(PpuMaskRegisterFlags::ShowSprites);

        if show_background {
            self.screen_state.bg_shift_pattern_lo <<= 1;
            self.screen_state.bg_shift_pattern_hi <<= 1;
            self.screen_state.bg_shift_attribute_lo <<= 1;
            self.screen_state.bg_shift_attribute_hi <<= 1;
        }

        if show_sprites && self.cycles >= 1 && self.cycles < 258 {
            let sprite_count = self.screen_state.sprite_count as usize;

            for (index, sprite) in self.internal_oam.chunks_mut(4).take(sprite_count).enumerate() {
                if sprite[3] > 0 {
                    sprite[3] -= 1;
                } else {
                    self.screen_state.sprite_shift_pattern_lo[index] <<= 1;
                    self.screen_state.sprite_shift_pattern_hi[index] <<= 1;
                }
            }
        }
    }

    fn fetch_data(&mut self) {
        let visible_scanline = self.scanline >= -1 && self.scanline < 240;

        match self.cycles {
            cycles @ (2..=257 | 321..=337) if visible_scanline => {
                self.update_shift();

                match (cycles - 1) % 8 {
                    0 => {
                        self.load_background_shift();
                        self.read_tile_id();
                    },
                    2 => self.read_attribute(),
                    4 => self.read_tile_lsb(),
                    6 => self.read_tile_msb(),
                    7 => self.increment_scroll_x(),
                    _ => (),
                }
            },
            _ => (),
        }
    }

    pub fn skip_odd_frame(&mut self) {
        if self.scanline == 0 && self.cycles == 0 {
            self.cycles = 1;
        }
    }

    pub fn reset_vblank(&mut self) {
        if self.scanline == -1 && self.cycles == 1 {
            self.status.set_flag(PpuStatusRegisterFlags::VBlank, false);
            self.status.set_flag(PpuStatusRegisterFlags::SpriteZeroHit, false);
            self.status.set_flag(PpuStatusRegisterFlags::SpriteOverflow, false);

            self.screen_state.sprite_shift_pattern_lo.fill(0);
            self.screen_state.sprite_shift_pattern_hi.fill(0);
        }
    }

    pub fn update_vblank(&mut self) {
        if self.scanline == 241 && self.cycles == 1 {
            self.status.set_flag(PpuStatusRegisterFlags::VBlank, true);

            if self.controller.get_flag(PpuControllerRegisterFlags::GenerateVBlankNMI) {
                self.bus.borrow_mut().set_interrupt(Some(()));
            }
        }
    }

    pub fn tick(&mut self, amount: usize) {
        self.cycles += amount;

        if self.scanline >= -1 && self.scanline < 240 {
            self.skip_odd_frame();
            self.reset_vblank();
            self.fetch_data();

            match self.cycles {
                256 => {
                    self.increment_scroll_y();
                },
                257 => {
                    self.load_background_shift();
                    self.transfer_address_x();

                    if self.scanline >= 0 {
                        self.internal_oam.fill(0xFF);
                        self.screen_state.sprite_count = 0;
                        self.screen_state.sprite_zero_occured = false;
                        self.screen_state.sprite_shift_pattern_lo.fill(0);
                        self.screen_state.sprite_shift_pattern_hi.fill(0);

                        self.bus
                            .borrow_mut()
                            .ppu_memory_map()
                            .get_oam()
                            .chunks(4)
                            .enumerate()
                            .for_each(|(index, sprite)| {
                                let sprite_count = self.screen_state.sprite_count;

                                if sprite_count < 9 {
                                    let diff = self.scanline - sprite[0] as isize;

                                    // TODO: 8x16 sprites
                                    if (0..8).contains(&diff) && sprite_count < 8 {
                                        if index == 0 {
                                            self.screen_state.sprite_zero_occured = true;
                                        }

                                        let internal_index = sprite_count as usize * 4;
                                        self.internal_oam[internal_index..internal_index + 4].copy_from_slice(sprite);
                                        self.screen_state.sprite_count += 1;
                                    }
                                }
                            });

                        self.status.set_flag(
                            PpuStatusRegisterFlags::SpriteOverflow,
                            self.screen_state.sprite_count > 8
                        );
                    }
                },
                280..=304 if self.scanline == -1 => {
                    self.transfer_address_y();
                },
                cycles @ (338 | 340) => {
                    self.read_tile_id();

                    if cycles == 340 && self.scanline >= 0 {
                        let sprite_count = self.screen_state.sprite_count as usize;
                        let sprite_pattern_table = if self.controller.get_flag(PpuControllerRegisterFlags::SpritesPatternTable) {
                            1u16
                        } else {
                            0u16
                        };

                        for (index, sprite) in self.internal_oam.chunks(4).take(sprite_count).enumerate() {
                            let pattern_address_lo = if sprite[2] & 0x80 != 0x80 {
                                (sprite_pattern_table << 12) |
                                ((sprite[1] as u16) << 4) |
                                (self.scanline - sprite[0] as isize) as u16
                            } else {
                                (sprite_pattern_table << 12) |
                                ((sprite[1] as u16) << 4) |
                                (7 - (self.scanline - sprite[0] as isize) as u16)
                            };

                            let pattern_address_hi = pattern_address_lo + 8;
                            let mut pattern_bits_lo = self.read(pattern_address_lo);
                            let mut pattern_bits_hi = self.read(pattern_address_hi);

                            if sprite[2] & 0x40 == 0x40 {
                                let flip_byte = |mut b| {
                                    b = (b & 0xF0) >> 4 | (b & 0x0F) << 4;
                                    b = (b & 0xCC) >> 2 | (b & 0x33) << 2;
                                    b = (b & 0xAA) >> 1 | (b & 0x55) << 1;
                                    b
                                };

                                pattern_bits_lo = flip_byte(pattern_bits_lo);
                                pattern_bits_hi = flip_byte(pattern_bits_hi);
                            }

                            self.screen_state.sprite_shift_pattern_lo[index] = pattern_bits_lo;
                            self.screen_state.sprite_shift_pattern_hi[index] = pattern_bits_hi;
                        }
                    }
                },
                _ => (),
            }
        }

        self.update_vblank();

        let mut bg_pixel = 0u8;
        let mut bg_palette = 0u8;

        let show_background = self.mask.get_flag(PpuMaskRegisterFlags::ShowBackground);
        let show_sprites = self.mask.get_flag(PpuMaskRegisterFlags::ShowSprites);

        if show_background {
            let bit_mux = 0x8000 >> self.fine_x;

            let p0_pixel = (self.screen_state.bg_shift_pattern_lo & bit_mux) > 0;
            let p1_pixel = (self.screen_state.bg_shift_pattern_hi & bit_mux) > 0;

            bg_pixel = (u8::from(p1_pixel) << 1) | u8::from(p0_pixel);

            let bg_palette0 = (self.screen_state.bg_shift_attribute_lo & bit_mux) > 0;
            let bg_palette1 = (self.screen_state.bg_shift_attribute_hi & bit_mux) > 0;

            bg_palette = (u8::from(bg_palette1) << 1) | u8::from(bg_palette0);
        }

        let mut fg_pixel = 0u8;
        let mut fg_palette = 0u8;
        let mut fg_priority = false;

        if show_sprites {
            self.screen_state.sprite_zero_rendering = false;

            let sprite_count = self.screen_state.sprite_count as usize;

            for (index, sprite) in self.internal_oam.chunks(4).take(sprite_count).enumerate() {
                if sprite[3] == 0 {
                    let pattern_lo = self.screen_state.sprite_shift_pattern_lo[index];
                    let pattern_hi = self.screen_state.sprite_shift_pattern_hi[index];

                    let fg_pixel_lo = u8::from((pattern_lo & 0x80) > 0);
                    let fg_pixel_hi = u8::from((pattern_hi & 0x80) > 0);

                    fg_pixel = (u8::from(fg_pixel_hi) << 1) | u8::from(fg_pixel_lo);

                    fg_palette = (sprite[2] & 0x03) + 0x04;
                    fg_priority = (sprite[2] & 0x20) == 0;

                    if fg_pixel != 0 {
                        if index == 0 {
                            self.screen_state.sprite_zero_rendering = true;
                        }

                        break;
                    }
                }
            }
        }

        let is_sprite_zero_hit = 
            self.screen_state.sprite_zero_occured &&
            self.screen_state.sprite_zero_rendering;

        let is_showing_leftmost = !(
            self.mask.get_flag(PpuMaskRegisterFlags::ShowBackgroundLeftmost) |
            self.mask.get_flag(PpuMaskRegisterFlags::ShowSpritesLeftmost)
        );

        let (pixel, palette) = match (bg_pixel, fg_pixel) {
            (0, 0) => (0x00, 0x00),
            (0, 1..=3) => (fg_pixel, fg_palette),
            (1..=3, 0) => (bg_pixel, bg_palette),
            (1..=3, 1..=3) => {
                if is_sprite_zero_hit && show_background && show_sprites {
                    if is_showing_leftmost {
                        if self.cycles >= 9 && self.cycles < 258 {
                            self.status.set_flag(PpuStatusRegisterFlags::SpriteZeroHit, true);
                        }
                    } else if self.cycles >= 1 && self.cycles < 258 {
                        self.status.set_flag(PpuStatusRegisterFlags::SpriteZeroHit, true);
                    }
                }
                if fg_priority {
                    (fg_pixel, fg_palette)
                } else {
                    (bg_pixel, bg_palette)
                }
            },
            _ => panic!("Invalid pixel data!"),
        };

        let pixel_color = self.read(0x3F00 + ((palette << 2) + pixel) as u16);

        self.screen_buffer.set_pixel(self.cycles - 1, self.scanline as usize, pixel_color);

        if self.cycles >= 341 {
            self.cycles = 0;
            self.scanline += 1;

            if self.scanline >= 261 {
                self.scanline = -1;
            }
        }
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
        self.controller.set(data);

        let nametable_x = data;
        let nametable_y = data >> 1;
        
        self.vram_temp.set_nametable_x(nametable_x as u16);
        self.vram_temp.set_nametable_y(nametable_y as u16);
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

        self.oamdata.set(data);
        self.oamaddress.set(oamaddress.wrapping_add(1));
    }

    pub fn write_scroll(&mut self, data: u8) {
        match self.address_latch {
            false => {
                let coarse_x = data >> 3;
                let fine_x = data & 0b111;

                self.vram_temp.set_coarse_x(coarse_x as u16);
                self.fine_x = fine_x as u16;
                self.address_latch = true;
            },
            true => {
                let coarse_y = data >> 3;
                let fine_y = data & 0b111;

                self.vram_temp.set_coarse_y(coarse_y as u16);
                self.vram_temp.set_fine_y(fine_y as u16);
                self.address_latch = false;
            },
        }
    }

    pub fn write_address(&mut self, data: u8) {
        match self.address_latch {
            false => {
                let [lo, _] = self.vram_temp.get().to_le_bytes();
                let vram = u16::from_le_bytes([lo, data]);

                self.vram_temp.set(vram);
                self.address_latch = true;
            },
            true => {
                let [_, hi] = self.vram_temp.get().to_le_bytes();
                let vram = u16::from_le_bytes([data, hi]);

                self.vram_temp.set(vram);
                self.vram.set(vram);
                self.address_latch = false;
            },
        }
    }

    pub fn write_data(&mut self, data: u8) {
        let address_increment = self.controller.get_flag(PpuControllerRegisterFlags::AddressIncrement);
        let address = self.vram.get();

        let write_address = match address {
            0x2000..=0x2FFF => self.mirror_address(address),
            0x3000..=0x3EFF => self.mirror_address(address - 0x1000),
            _ => address,
        };

        self.write(write_address, data);
        self.data.set(data);
        self.vram.set(if address_increment {
            address.wrapping_add(32)
        } else {
            address.wrapping_add(1)
        });
    }

    pub fn read_status(&mut self) -> u8 {
        let result = (self.status.get() & 0xE0) | (self.internal_buf.unwrap_or(0) & 0x1F);

        self.status.set_flag(PpuStatusRegisterFlags::VBlank, false);
        self.address_latch = false;

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
        let address = self.vram.get();
        let read_address = match address {
            0x2000..=0x2FFF => self.mirror_address(address),
            0x3000..=0x3EFF => self.mirror_address(address - 0x1000),
            _ => address,
        };

        self.vram.set(if address_increment {
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
