use super::cartridge::Cartridge;
use super::memorymap::CpuMemoryMap;
use super::memorymap::PpuMemoryMap;

pub struct Bus {
    cpu_memory_map: Box<CpuMemoryMap>,
    ppu_memory_map: Box<PpuMemoryMap>,
    nmi_interrupt: Option<()>,
}

impl Bus {
    pub fn new(cartridge: &Cartridge) -> Self {
        Self {
            cpu_memory_map: Box::new(CpuMemoryMap::new(cartridge.get_mapper())),
            ppu_memory_map: Box::new(PpuMemoryMap::new(cartridge.get_mapper())),
            nmi_interrupt: None,
        }
    }

    pub fn cpu_memory_map(&mut self) -> &mut Box<CpuMemoryMap> {
        &mut self.cpu_memory_map
    }

    pub fn ppu_memory_map(&mut self) -> &mut Box<PpuMemoryMap> {
        &mut self.ppu_memory_map
    }

    pub fn set_interrupt(&mut self, interrupt: Option<()>) {
        self.nmi_interrupt = interrupt;
    }

    pub fn get_interrupt(&self) -> &Option<()> {
        &self.nmi_interrupt
    }

    pub fn poll_interrupt(&mut self) -> Option<()> {
        self.nmi_interrupt.take()
    }
}
