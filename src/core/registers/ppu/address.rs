use crate::core::registers::Register;

#[repr(u8)]
pub enum PpuAddressRegisterFlags {
    _Bit0 = 1 << 0,
    _Bit1 = 1 << 1,
    _Bit2 = 1 << 2,
    _Bit3 = 1 << 3,
    _Bit4 = 1 << 4,
    _Bit5 = 1 << 5,
    _Bit6 = 1 << 6,
    _Bit7 = 1 << 7,
}

pub struct PpuAddressRegister {
    address: (u8, u8),
    latch: bool,
}

impl PpuAddressRegister {
    pub fn new() -> Self {
        Self {
            address: (0x00, 0x00),
            latch: false,
        }
    }

    fn fix_overflow(&mut self) {
        let address = u16::from_le_bytes([self.address.1, self.address.0]) & 0x3FFF;
        let [lo, hi] = address.to_le_bytes();

        self.address.0 = hi;
        self.address.1 = lo;
    }

    pub fn get_address(&self) -> u16 {
        u16::from_le_bytes([self.address.1, self.address.0])
    }

    pub fn set_address(&mut self, address: u16) {
        let [lo, hi] = address.to_le_bytes();

        self.address = (hi, lo);
        self.fix_overflow();
    }

    pub fn reset_latch(&mut self) {
        self.latch = false;
    }
}

impl Register<PpuAddressRegisterFlags, u8> for PpuAddressRegister {
    fn get(&self) -> u8 {
        if self.latch {
            self.address.1
        } else {
            self.address.0
        }
    }

    fn set(&mut self, value: u8) {
        if self.latch {
            self.address.1 = value;
        } else {
            self.address.0 = value;
        }

        self.fix_overflow();

        self.latch = !self.latch;
    }

    fn get_flag(&self, flag: PpuAddressRegisterFlags) -> bool {
        let value = if self.latch {
            self.address.1
        } else {
            self.address.0
        };

        value & flag as u8 != 0
    }

    fn set_flag(&mut self, flag: PpuAddressRegisterFlags, active: bool) {
        let value = if self.latch {
            &mut self.address.1
        } else {
            &mut self.address.0
        };

        if active {
            *value |= flag as u8;
        } else {
            *value &= !(flag as u8);
        }
    }
}
