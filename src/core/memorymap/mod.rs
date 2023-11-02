pub mod cpu;
pub use cpu::*;

pub mod ppu;
pub use ppu::*;

use super::memory::Memory;

pub enum MemoryMapType {
    Cpu,
    Ppu,
}

pub trait MemoryMap: Memory {
    fn get_type(&self) -> MemoryMapType;
}
