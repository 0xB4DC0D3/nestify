pub mod cpu;
pub use cpu::*;

pub mod ppu;
pub use ppu::*;

use super::memory::Memory;

pub trait MemoryMap: Memory {}
