use crate::core::registers::Register;

#[repr(u8)]
pub enum PpuStatusRegisterFlags {
    SpriteZeroHit = 1 << 6,
    VBlank = 1 << 7,
}

pub struct PpuStatusRegister {
    value: u8
}

impl PpuStatusRegister {
    pub fn new() -> Self {
        // Break and InterruptDisable always true when initialized
        Self {
            value: 0x00,
        }
    }
}

impl Register<PpuStatusRegisterFlags, u8> for PpuStatusRegister {
    fn get(&self) -> u8 {
        self.value
    }

    fn set(&mut self, value: u8) {
        self.value = value
    }

    fn get_flag(&self, flag: PpuStatusRegisterFlags) -> bool {
        self.value & flag as u8 != 0
    }

    fn set_flag(&mut self, flag: PpuStatusRegisterFlags, active: bool) {
        if active {
            self.value |= flag as u8;
        } else {
            self.value &= !(flag as u8);
        }
    }
}
