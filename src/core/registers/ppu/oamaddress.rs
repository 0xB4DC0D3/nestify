use crate::core::registers::Register;

#[repr(u8)]
pub enum PpuOamAddressRegisterFlags {
    _Bit0 = 1 << 0,
    _Bit1 = 1 << 1,
    _Bit2 = 1 << 2,
    _Bit3 = 1 << 3,
    _Bit4 = 1 << 4,
    _Bit5 = 1 << 5,
    _Bit6 = 1 << 6,
    _Bit7 = 1 << 7,
}

pub struct PpuOamAddressRegister {
    value: u8
}

impl PpuOamAddressRegister {
    pub fn new() -> Self {
        Self {
            value: 0x00,
        }
    }
}

impl Register<PpuOamAddressRegisterFlags, u8> for PpuOamAddressRegister {
    fn get(&self) -> u8 {
        self.value
    }

    fn set(&mut self, value: u8) {
        self.value = value
    }

    fn get_flag(&self, flag: PpuOamAddressRegisterFlags) -> bool {
        self.value & flag as u8 != 0
    }

    fn set_flag(&mut self, flag: PpuOamAddressRegisterFlags, active: bool) {
        if active {
            self.value |= flag as u8;
        } else {
            self.value &= !(flag as u8);
        }
    }
}
