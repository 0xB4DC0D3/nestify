use super::memory::Memory;

pub struct MemoryMap<const S: usize> {
    memory: [u8; S],
}

impl<const S: usize> MemoryMap<S> {
    pub fn new() -> Self {
        Self {
            memory: [0u8; S],
        }
    }
}

impl<const S: usize> Memory for MemoryMap<S> {
    fn read(&self, address: u16) -> u8 {
        self.memory[address as usize]
    }

    fn write(&mut self, address: u16, data: u8) {
        self.memory[address as usize] = data;
    }
}
