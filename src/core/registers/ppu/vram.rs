pub struct PpuVRamRegister {
    coarse_x: u16,
    coarse_y: u16,
    nametable_x: u16,
    nametable_y: u16,
    fine_y: u16,
}

impl PpuVRamRegister {
    pub fn new() -> Self {
        Self {
            coarse_x: 0x0000,
            coarse_y: 0x0000,
            nametable_x: 0x0000,
            nametable_y: 0x0000,
            fine_y: 0x0000,
        }
    }

    fn update_bits(&mut self, address: u16) {
        self.coarse_x = address & 0b11111;
        self.coarse_y = (address >> 5) & 0b11111;
        self.nametable_x = (address >> 10) & 0b1;
        self.nametable_y = (address >> 11) & 0b1;
        self.fine_y = (address >> 12) & 0b111;
    }

    pub fn set(&mut self, address: u16) {
        self.update_bits(address & 0x3FFF);
    }

    pub fn get(&mut self) -> u16 {
        (
            ((self.fine_y & 0b111) << 12) |
            ((self.nametable_y & 0b1) << 11) |
            ((self.nametable_x & 0b1) << 10) |
            ((self.coarse_y & 0b11111) << 5) |
            (self.coarse_x & 0b11111)
        ) & 0x3FFF
    }

    pub fn set_coarse_x(&mut self, coarse_x: u16) {
        self.coarse_x = coarse_x & 0b11111;
    }

    pub fn get_coarse_x(&self) -> u16 {
        self.coarse_x & 0b11111
    }

    pub fn set_coarse_y(&mut self, coarse_y: u16) {
        self.coarse_y = coarse_y & 0b11111;
    }

    pub fn get_coarse_y(&self) -> u16 {
        self.coarse_y & 0b11111
    }

    pub fn set_nametable_x(&mut self, nametable_x: u16) {
        self.nametable_x = nametable_x & 0b1;
    }

    pub fn get_nametable_x(&self) -> u16 {
        self.nametable_x & 0b1
    }

    pub fn set_nametable_y(&mut self, nametable_y: u16) {
        self.nametable_y = nametable_y & 0b1;
    }

    pub fn get_nametable_y(&self) -> u16 {
        self.nametable_y & 0b1
    }

    pub fn set_fine_y(&mut self, fine_y: u16) {
        self.fine_y = fine_y & 0b111;
    }

    pub fn get_fine_y(&self) -> u16 {
        self.fine_y & 0b111
    }
}
