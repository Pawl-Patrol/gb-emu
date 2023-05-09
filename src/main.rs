mod cartridge;
mod constants;
mod cpu;
mod emulator;
mod mmu;
mod opcodes;
mod traits;

use cartridge::Cartridge;
use emulator::Emulator;

fn main() {
    let mut cartridge = Cartridge::new();
}
