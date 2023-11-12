pub struct PpuVRamRegister {
    address: u16,
}

impl PpuVRamRegister {
    pub fn new() -> Self {
        Self {
            address: 0x0000,
        }
    }

    pub fn get(&self) -> u16 {
        self.address
    }

    pub fn set(&mut self, value: u16) {
        self.address = value;
        self.address &= 0x3FFF;
    }

    pub fn set_coarse_x(&mut self, coarse_x: u16) {
        let mask = 0b0000_0000_0001_1111;
        let value = (coarse_x << 0) & mask;
        let address = self.address & !mask & 0x3FFF;

        self.address = address | value;
    }

    pub fn get_coarse_x(&self) -> u16 {
        self.address & 0b11111
    }

    pub fn set_coarse_y(&mut self, coarse_y: u16) {
        let mask = 0b0000_0011_1110_0000;
        let value = (coarse_y << 5) & mask;
        let address = self.address & !mask & 0x3FFF;

        self.address = address | value;
    }

    pub fn get_coarse_y(&self) -> u16 {
        (self.address >> 5) & 0b11111
    }

    pub fn set_nametable_x(&mut self, nametable_x: u16) {
        let mask = 0b0000_0100_0000_0000;
        let value = ((nametable_x & 0b1) << 10) & mask;
        let address = self.address & !mask & 0x3FFF;
        
        self.address = address | value;
    }

    pub fn get_nametable_x(&self) -> u16 {
        (self.address >> 10) & 0b1
    }

    pub fn set_nametable_y(&mut self, nametable_y: u16) {
        let mask = 0b0000_1000_0000_0000;
        let value = ((nametable_y & 0b1) << 11) & mask;
        let address = self.address & !mask & 0x3FFF;

        self.address = address | value;
    }

    pub fn get_nametable_y(&self) -> u16 {
        (self.address >> 11) & 0b1
    }

    pub fn set_fine_y(&mut self, fine_y: u16) {
        let mask = 0b0111_0000_0000_0000;
        let value = (fine_y << 12) & mask;
        let address = self.address & !mask & 0x3FFF;

        self.address = address | value;
    }

    pub fn get_fine_y(&self) -> u16 {
        (self.address >> 12) & 0b111
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_setters() {
        let mut vram = PpuVRamRegister::new();

        vram.set_coarse_x(0b11110);
        assert_eq!(vram.get_coarse_x(), 0b11110, "Invalid coarse X value!");

        vram.set_coarse_y(0b11110);
        assert_eq!(vram.get_coarse_y(), 0b11110, "Invalid coarse Y value!");
        assert_eq!(vram.get(), 0b1111011110, "Invalid VRAM address!");
        
        vram.set_nametable_x(1);
        assert_eq!(vram.get_nametable_x(), 0b1, "Invalid nametable value!");
        assert_eq!(vram.get(), 0b11111011110, "Invalid VRAM address!");

        vram.set_nametable_x(!vram.get_nametable_x());
        assert_eq!(vram.get_nametable_x(), 0b0, "Invalid nametable value!");
        assert_eq!(vram.get(), 0b01111011110, "Invalid VRAM address!");

        vram.set_fine_y(0b110);
        assert_eq!(vram.get_fine_y(), 0b110, "Invalid fine Y value!");
        assert_eq!(vram.get(), 0b110001111011110, "Invalid VRAM address!");
    }
}
