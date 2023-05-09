mod cartridge;
mod constants;
mod cpu;
mod emulator;
mod mmu;
mod opcodes;
mod traits;

use emulator::Emulator;

fn call_fps(fps: f32, f: &mut dyn FnMut()) {
    let frame_time = 1.0 / fps;
    let mut last_frame = std::time::Instant::now();
    loop {
        let now = std::time::Instant::now();
        let elapsed = now.duration_since(last_frame).as_secs_f32();
        if elapsed >= frame_time {
            f();
            last_frame = now;
        }
    }
}

fn main() {
    let mut emulator = Emulator::new();
    // reset log file
    std::fs::write("./log.txt", "").unwrap();
    emulator.load_rom("./roms/11-op a,(hl).gb");
    // print cartridge section 0x100 to 0x150
    let mem = emulator.cartrige.rom[0x0213..0x0230].to_vec();
    call_fps(60.0, &mut || {
        emulator.update();
    })
}
