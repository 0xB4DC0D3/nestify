use std::rc::Rc;
use std::cell::RefCell;

use nestify::core::cpu::Cpu;
use nestify::core::clock::Clock;
use nestify::core::bus::Bus;
use nestify::core::cartridge::Cartridge;
use nestify::core::ppu::Ppu;

fn main() {
    let rom = std::fs::read("nestest.nes").expect("Unable to read `nestest.nes`!");
    let cartridge = Cartridge::new(rom);
    let bus = Rc::new(RefCell::new(Bus::new(&cartridge)));
    let ppu = Rc::new(RefCell::new(Ppu::new(&bus, cartridge.get_mirroring())));
    let clock = Rc::new(RefCell::new(Clock::new(&ppu)));
    let mut cpu = Cpu::new(&bus, &clock);

    cpu.use_disassembler(true);
    cpu.set_program_counter(0xC000);

    loop {
        cpu.fetch();
    }
}
