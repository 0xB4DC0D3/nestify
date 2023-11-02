use super::cartridge::Cartridge;
use super::memorymap::MemoryMap;
use super::memorymap::CpuMemoryMap;
use super::memorymap::PpuMemoryMap;
use super::memorymap::MemoryMapType;

pub struct Bus {
    cpu_memory_map: Box<dyn MemoryMap>,
    ppu_memory_map: Box<dyn MemoryMap>,
    nmi_interrupt: Option<()>,
}

impl Bus {
    pub fn new(cartridge: &Cartridge) -> Self {
        Self {
            cpu_memory_map: Box::new(CpuMemoryMap::new(cartridge.get_mapper())),
            ppu_memory_map: Box::new(PpuMemoryMap::new()),
            nmi_interrupt: None,
        }
    }

    pub fn get_memory_map(&mut self, memory_map_type: MemoryMapType) -> &mut Box<dyn MemoryMap> {
        match memory_map_type {
            MemoryMapType::Cpu => &mut self.cpu_memory_map,
            MemoryMapType::Ppu => &mut self.ppu_memory_map,
        }
    }

    pub fn set_interrupt(&mut self, interrupt: Option<()>) {
        self.nmi_interrupt = interrupt;
    }

    pub fn poll_interrupt(&mut self) -> Option<()> {
        self.nmi_interrupt.take()
    }
}
