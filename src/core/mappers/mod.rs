pub mod mapper000;
pub use mapper000::*;

use super::memory::Memory;

pub trait Mapper: Memory {
    fn get_chr_rom(&mut self) -> &mut Vec<u8>;
}
