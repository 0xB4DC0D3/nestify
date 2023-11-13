use crate::core::registers::Register;

#[repr(u8)]
pub enum PpuMaskRegisterFlags {
    _Greyscale = 1 << 0,
    ShowBackgroundLeftmost = 1 << 1,
    ShowSpritesLeftmost = 1 << 2,
    ShowBackground = 1 << 3,
    ShowSprites = 1 << 4,
    _EmphasizeRed = 1 << 5,
    _EmphasizeGreen = 1 << 6,
    _EmphasizeBlue = 1 << 7,
}

pub struct PpuMaskRegister {
    value: u8
}

impl PpuMaskRegister {
    pub fn new() -> Self {
        Self {
            value: 0x00,
        }
    }
}

impl Register<PpuMaskRegisterFlags, u8> for PpuMaskRegister {
    fn get(&self) -> u8 {
        self.value
    }

    fn set(&mut self, value: u8) {
        self.value = value
    }

    fn get_flag(&self, flag: PpuMaskRegisterFlags) -> bool {
        self.value & flag as u8 != 0
    }

    fn set_flag(&mut self, flag: PpuMaskRegisterFlags, active: bool) {
        if active {
            self.value |= flag as u8;
        } else {
            self.value &= !(flag as u8);
        }
    }
}
