use crate::core::registers::Register;

#[repr(u8)]
pub enum PpuOamAddressRegisterFlags {
    Bit0 = 1 << 0,
    Bit1 = 1 << 1,
    Bit2 = 1 << 2,
    Bit3 = 1 << 3,
    Bit4 = 1 << 4,
    Bit5 = 1 << 5,
    Bit6 = 1 << 6,
    Bit7 = 1 << 7,
}

pub struct PpuOamAddressRegister {
    value: u8
}

impl PpuOamAddressRegister {
    pub fn new() -> Self {
        // Break and InterruptDisable always true when initialized
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
