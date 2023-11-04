use std::cell::RefCell;
use std::rc::Rc;

use super::bus::Bus;
use super::memory::Memory;
use super::registers::Register;
use super::registers::ppu::data::PpuDataRegister;
use super::registers::ppu::address::PpuAddressRegister;
use super::registers::ppu::scroll::PpuScrollRegister;
use super::registers::ppu::oamdata::PpuOamDataRegister;
use super::registers::ppu::oamaddress::PpuOamAddressRegister;
use super::registers::ppu::status::{PpuStatusRegister, PpuStatusRegisterFlags};
use super::registers::ppu::mask::PpuMaskRegister;
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

    pub fn tick(&mut self, amount: usize) {
        self.cycles += amount * 3;
    }

    pub fn mirror_address(&mut self, address: u16) -> u16 {
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
        let start = u16::from_le_bytes([data, 0x00]);
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
