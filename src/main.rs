mod canvas;
mod cartridge;
mod constants;
mod cpu;
mod emulator;
mod mmu;
mod opcodes;
mod traits;

use emulator::Emulator;
use minifb::Window;

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

fn run_rom(path: &str) {
    let mut window = Window::new(
        "Gameboy Emulator",
        constants::SCREEN_WIDTH,
        constants::SCREEN_HEIGHT,
        minifb::WindowOptions::default(),
    )
    .unwrap();
    let mut emulator = Emulator::new();
    emulator.load_rom(path);

    window.limit_update_rate(Some(std::time::Duration::from_secs_f32(1.0 / 60.0)));
    // keyboard event
    let mut last_frame = std::time::Instant::now();
    loop {
        let now = std::time::Instant::now();
        let elapsed = now.duration_since(last_frame).as_secs_f32();

        if window.is_key_down(minifb::Key::Escape) {
            std::process::exit(0);
        }
        if window.is_key_down(minifb::Key::Up) {
            emulator.on_key_pressed(constants::KEY_UP);
        }
        if window.is_key_down(minifb::Key::Down) {
            emulator.on_key_pressed(constants::KEY_DOWN);
        }
        if window.is_key_down(minifb::Key::Left) {
            emulator.on_key_pressed(constants::KEY_LEFT);
        }
        if window.is_key_down(minifb::Key::Right) {
            emulator.on_key_pressed(constants::KEY_RIGHT);
        }
        if window.is_key_down(minifb::Key::A) {
            emulator.on_key_pressed(constants::KEY_A);
        }
        if window.is_key_down(minifb::Key::B) {
            emulator.on_key_pressed(constants::KEY_B);
        }
        if window.is_key_down(minifb::Key::Enter) {
            println!("start");
            emulator.on_key_pressed(constants::KEY_START);
        }
        if window.is_key_down(minifb::Key::Space) {
            emulator.on_key_pressed(constants::KEY_SELECT);
        }
        if window.is_key_released(minifb::Key::Up) {
            emulator.on_key_released(constants::KEY_UP);
        }
        if window.is_key_released(minifb::Key::Down) {
            emulator.on_key_released(constants::KEY_DOWN);
        }
        if window.is_key_released(minifb::Key::Left) {
            emulator.on_key_released(constants::KEY_LEFT);
        }
        if window.is_key_released(minifb::Key::Right) {
            emulator.on_key_released(constants::KEY_RIGHT);
        }
        if window.is_key_released(minifb::Key::A) {
            emulator.on_key_released(constants::KEY_A);
        }
        if window.is_key_released(minifb::Key::B) {
            emulator.on_key_released(constants::KEY_B);
        }
        if window.is_key_released(minifb::Key::Enter) {
            emulator.on_key_released(constants::KEY_START);
        }
        if window.is_key_released(minifb::Key::Space) {
            emulator.on_key_released(constants::KEY_SELECT);
        }
        for _ in 0..69905 {
            emulator.update();
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

fn run_test_rom(path: &str) {
    let mut emulator = Emulator::new();
    // reset log file
    std::fs::write("./log.txt", "").unwrap();
    emulator.load_rom(path);
    loop {
        emulator.update();
    }
}

fn main() {
    run_rom("./roms/pokemon.gb");
}
