use crate::core::registers::Register;

#[repr(u8)]
pub enum PpuScrollRegisterFlags {
    _Bit0 = 1 << 0,
    _Bit1 = 1 << 1,
    _Bit2 = 1 << 2,
    _Bit3 = 1 << 3,
    _Bit4 = 1 << 4,
    _Bit5 = 1 << 5,
    _Bit6 = 1 << 6,
    _Bit7 = 1 << 7,
}

pub struct PpuScrollRegister {
    value: (u8, u8),
    latch: bool,
}

impl PpuScrollRegister {
    pub fn new() -> Self {
        Self {
            value: (0x00, 0x00),
            latch: false,
        }
    }

    pub fn reset_latch(&mut self) {
        self.latch = false;
    }

    pub fn get_scroll(&self) -> (u8, u8) {
        self.value
    }
}

impl Register<PpuScrollRegisterFlags, u8> for PpuScrollRegister {
    fn get(&self) -> u8 {
        if self.latch {
            self.value.1
        } else {
            self.value.0
        }
    }

    fn set(&mut self, value: u8) {
        if self.latch {
            self.value.1 = value;
        } else {
            self.value.0 = value;
        }

        self.latch = !self.latch;
    }

    fn get_flag(&self, flag: PpuScrollRegisterFlags) -> bool {
        let value = if self.latch {
            self.value.1
        } else {
            self.value.0
        };

        value & flag as u8 != 0
    }

    fn set_flag(&mut self, flag: PpuScrollRegisterFlags, active: bool) {
        let value = if self.latch {
            &mut self.value.1
        } else {
            &mut self.value.0
        };

        if active {
            *value |= flag as u8;
        } else {
            *value &= !(flag as u8);
        }
    }
}
