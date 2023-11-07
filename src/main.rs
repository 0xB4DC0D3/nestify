use std::rc::Rc;
use std::cell::RefCell;

use nestify::core::cpu::Cpu;
use nestify::core::clock::Clock;
use nestify::core::bus::Bus;
use nestify::core::cartridge::Cartridge;
use nestify::core::ppu::Ppu;
use nestify::gui::window::Window;
use sdl2::event::Event;

fn main() {
    let mut window = Window::new();
    let rom = std::fs::read("super_mario.nes").expect("Unable to read `nestest.nes`!");
    let cartridge = Cartridge::new(rom);
    let bus = Rc::new(RefCell::new(Bus::new(&cartridge)));
    let ppu = Rc::new(RefCell::new(Ppu::new(&bus, cartridge.get_mirroring())));
    let clock = Rc::new(RefCell::new(Clock::new(&ppu, move |ppu| {
        window.render(ppu);
        
        for event in window.event_pump().poll_iter() {
            match event {
                Event::Quit { .. } => {
                    std::process::exit(0);
                },
                _ => (),
            }
        }
    })));

    let mut cpu = Cpu::new(&bus, &clock);

    // cpu.use_disassembler(true);
    // cpu.set_program_counter(0xC000);
    cpu.reset();

    loop {
        cpu.fetch();
    }
}
