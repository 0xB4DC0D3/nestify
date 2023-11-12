use crate::core::registers::Register;

#[repr(u8)]
pub enum PpuControllerRegisterFlags {
    _ScrollX = 1 << 0,
    _ScrollY = 1 << 1,
    AddressIncrement = 1 << 2,
    SpritesPatternTable = 1 << 3,
    BackgroundPatternTable = 1 << 4,
    _SpriteSize = 1 << 5,
    _MasterSlaveSelect = 1 << 6,
    GenerateVBlankNMI = 1 << 7,
}

pub struct PpuControllerRegister {
    value: u8
}

impl PpuControllerRegister {
    pub fn new() -> Self {
        Self {
            value: 0x00,
        }
    }
}

impl Register<PpuControllerRegisterFlags, u8> for PpuControllerRegister {
    fn get(&self) -> u8 {
        self.value
    }

    fn set(&mut self, value: u8) {
        self.value = value
    }

    fn get_flag(&self, flag: PpuControllerRegisterFlags) -> bool {
        self.value & flag as u8 != 0
    }

    fn set_flag(&mut self, flag: PpuControllerRegisterFlags, active: bool) {
        if active {
            self.value |= flag as u8;
        } else {
            self.value &= !(flag as u8);
        }
    }
}
