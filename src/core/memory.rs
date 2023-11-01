pub trait Memory {
    fn read(&self, address: u16) -> u8;
    fn read_u16(&self, address: u16) -> u16 {
        let lo = self.read(address);
        let hi = self.read(address.wrapping_add(1));

        u16::from_le_bytes([lo, hi])
    }

    fn write(&mut self, address: u16, data: u8);
    fn write_u16(&mut self, address: u16, data: u8) {
        self.write(address, data);
        self.write(address.wrapping_add(1), data);
    }
}
