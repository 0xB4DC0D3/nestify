use super::memorymap::MemoryMap;

pub struct Bus {
    cpu_memory_map: MemoryMap<0x10000>,
    ppu_memory_map: MemoryMap<0x4000>,
    nmi_interrupt: Option<()>,
}

impl Bus {
    pub fn new() -> Self {
        Self {
            cpu_memory_map: MemoryMap::new(),
            ppu_memory_map: MemoryMap::new(),
            nmi_interrupt: None,
        }
    }

    pub fn get_cpu_memory_map(&mut self) -> &mut MemoryMap<0x10000> {
        &mut self.cpu_memory_map
    }

    pub fn get_ppu_memory_map(&mut self) -> &mut MemoryMap<0x4000> {
        &mut self.ppu_memory_map
    }

    pub fn set_interrupt(&mut self, interrupt: Option<()>) {
        self.nmi_interrupt = interrupt;
    }

    pub fn poll_interrupt(&mut self) -> Option<()> {
        self.nmi_interrupt.take()
    }
}
