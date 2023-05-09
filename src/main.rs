mod cartridge;
mod cpu;
mod emulator;
mod gpu;
mod joypad;
mod mmu;
mod rtc;
mod traits;

use emulator::Emulator;
fn main() {
    let mut emulator = Emulator::new("./roms/pikachu.gb", "./saves/pikachu.sav");
    emulator.run();
}
