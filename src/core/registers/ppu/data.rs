use crate::core::registers::Register;

#[repr(u8)]
pub enum PpuDataRegisterFlags {
    _Bit0 = 1 << 0,
    _Bit1 = 1 << 1,
    _Bit2 = 1 << 2,
    _Bit3 = 1 << 3,
    _Bit4 = 1 << 4,
    _Bit5 = 1 << 5,
    _Bit6 = 1 << 6,
    _Bit7 = 1 << 7,
}

pub struct PpuDataRegister {
    value: u8
}

impl PpuDataRegister {
    pub fn new() -> Self {
        Self {
            value: 0x00,
        }
    }
}

impl Register<PpuDataRegisterFlags, u8> for PpuDataRegister {
    fn get(&self) -> u8 {
        self.value
    }

    fn set(&mut self, value: u8) {
        self.value = value
    }

    fn get_flag(&self, flag: PpuDataRegisterFlags) -> bool {
        self.value & flag as u8 != 0
    }

    fn set_flag(&mut self, flag: PpuDataRegisterFlags, active: bool) {
        if active {
            self.value |= flag as u8;
        } else {
            self.value &= !(flag as u8);
        }
    }
}
