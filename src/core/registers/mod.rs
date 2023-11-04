pub mod cpu;
pub mod ppu;

pub trait Register<F, V> {
    fn get(&self) -> V;
    fn set(&mut self, value: V);
    fn get_flag(&self, flag: F) -> bool;
    fn set_flag(&mut self, flag: F, active: bool);
}
