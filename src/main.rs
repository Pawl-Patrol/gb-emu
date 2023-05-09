mod canvas;
mod cartridge;
mod clock;
mod constants;
mod cpu;
mod emulator;
mod graphics;
mod input;
mod interrupts;
mod mmu;
mod opcodes;
mod traits;

use emulator::Emulator;
use minifb::Window;

fn run_rom(path: &str) {
    let mut window = Window::new(
        "Gameboy Emulator",
        constants::SCREEN_WIDTH,
        constants::SCREEN_HEIGHT,
        minifb::WindowOptions::default(),
    )
    .unwrap();
    window.limit_update_rate(Some(std::time::Duration::from_secs_f32(1.0 / 60.0)));

    let mut emulator = Emulator::new();
    emulator.load_rom(path);

    let key_mapping = vec![
        (minifb::Key::Up, constants::KEY_UP),
        (minifb::Key::Down, constants::KEY_DOWN),
        (minifb::Key::Left, constants::KEY_LEFT),
        (minifb::Key::Right, constants::KEY_RIGHT),
        (minifb::Key::A, constants::KEY_A),
        (minifb::Key::B, constants::KEY_B),
        (minifb::Key::Enter, constants::KEY_START),
        (minifb::Key::Space, constants::KEY_SELECT),
    ];

    let mut last = std::time::Instant::now();
    loop {
        let now = std::time::Instant::now();
        let elapsed = now.duration_since(last).as_micros();
        last = now;

        if window.is_key_down(minifb::Key::Escape) {
            std::process::exit(0);
        }
        for (from, to) in &key_mapping {
            if window.is_key_down(*from) {
                emulator.on_key_pressed(*to);
            } else if window.is_key_released(*from) {
                emulator.on_key_released(*to);
            }
        }

        let ticks = elapsed * 4194304 / 1000000;
        let mut cycles = 0;
        while cycles < ticks {
            cycles += emulator.update() as u128;
        }

        window
            .update_with_buffer(
                &emulator.video_buffer,
                constants::SCREEN_WIDTH,
                constants::SCREEN_HEIGHT,
            )
            .unwrap();
    }
}

fn main() {
    run_rom("./roms/pokemon.gb");
}
