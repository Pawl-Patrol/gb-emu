mod cartridge;
mod cpu;
mod gpu;
mod joypad;
mod mmu;
mod rtc;
mod traits;

use mini_gl_fb::glutin::dpi::LogicalSize;
use mini_gl_fb::glutin::event::VirtualKeyCode as Key;
use mini_gl_fb::glutin::event_loop::EventLoop;
use mini_gl_fb::{get_fancy, ConfigBuilder};

fn run_rom(path: &str) {
    let mut event_loop = EventLoop::new();
    let config = ConfigBuilder::default()
        .window_title("Gameboy Emulator".to_string())
        .buffer_size(Some(LogicalSize::new(
            gpu::SCREEN_WIDTH as u32,
            gpu::SCREEN_HEIGHT as u32,
        )))
        .resizable(true)
        .invert_y(false)
        .build();
    let mut fb = get_fancy(config, &event_loop);

    let mut cpu = cpu::CPU::new();
    cpu.mmu.load_rom(path);

    let key_mapping = vec![
        (Key::Up, joypad::KEY_UP),
        (Key::Down, joypad::KEY_DOWN),
        (Key::Left, joypad::KEY_LEFT),
        (Key::Right, joypad::KEY_RIGHT),
        (Key::A, joypad::KEY_A),
        (Key::B, joypad::KEY_B),
        (Key::Return, joypad::KEY_START),
        (Key::Space, joypad::KEY_SELECT),
    ];

    let mut previous = std::time::Instant::now();

    fb.glutin_handle_basic_input(&mut event_loop, |fb, input| {
        let now = std::time::Instant::now();
        let elapsed = now.duration_since(previous);
        previous = now;

        let t1 = std::time::Instant::now();
        if input.key_is_down(Key::Escape) {
            return false;
        }
        for (from, to) in &key_mapping {
            if input.key_pressed(*from) {
                cpu.mmu.interrupt_flag |= cpu.mmu.joypad.on_key_pressed(*to);
            } else if input.key_released(*from) {
                cpu.mmu.joypad.on_key_released(*to);
            }
        }

        let t2 = std::time::Instant::now();

        let ticks = elapsed.as_micros() * 4194304 / 1000000;
        let mut cycles = 0;

        while cycles < ticks {
            cycles += cpu.update() as u128;
        }

        let t3 = std::time::Instant::now();

        fb.update_buffer(&cpu.mmu.gpu.video_buffer);
        let t4 = std::time::Instant::now();

        println!(
            "joypad: {:?}, cpu: {:?}, window: {:?}",
            t2.duration_since(t1),
            t3.duration_since(t2),
            t4.duration_since(t3)
        );

        true
    })
}

fn main() {
    run_rom("./roms/pikachu.gb");
}
