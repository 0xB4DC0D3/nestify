use crate::core::registers::Register;

#[repr(u8)]
pub enum PpuOamDataRegisterFlags {
    Bit0 = 1 << 0,
    Bit1 = 1 << 1,
    Bit2 = 1 << 2,
    Bit3 = 1 << 3,
    Bit4 = 1 << 4,
    Bit5 = 1 << 5,
    Bit6 = 1 << 6,
    Bit7 = 1 << 7,
}

pub struct PpuOamDataRegister {
    value: u8
}

impl PpuOamDataRegister {
    pub fn new() -> Self {
        // Break and InterruptDisable always true when initialized
        Self {
            value: 0x00,
        }
    }
}

impl Register<PpuOamDataRegisterFlags, u8> for PpuOamDataRegister {
    fn get(&self) -> u8 {
        self.value
    }

    fn set(&mut self, value: u8) {
        self.value = value
    }

    fn get_flag(&self, flag: PpuOamDataRegisterFlags) -> bool {
        self.value & flag as u8 != 0
    }

    fn set_flag(&mut self, flag: PpuOamDataRegisterFlags, active: bool) {
        if active {
            self.value |= flag as u8;
        } else {
            self.value &= !(flag as u8);
        }
    }
}
