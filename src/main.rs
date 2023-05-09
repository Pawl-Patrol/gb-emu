mod canvas;
mod cartridge;
mod constants;
mod context;
mod cpu;
mod gpu;
mod joypad;
mod mmu;
mod rtc;
mod traits;

use std::{
    borrow::BorrowMut,
    cell::{Cell, RefCell, RefMut},
    rc::Rc,
};

use context::Context;
use cpu::CPU;
use gpu::GPU;
use joypad::JoyPad;
use minifb::Window;
use mmu::MMU;
use rtc::RTC;

fn run_rom(path: &str) {
    let mut window = Window::new(
        "Gameboy Emulator",
        constants::SCREEN_WIDTH,
        constants::SCREEN_HEIGHT,
        minifb::WindowOptions::default(),
    )
    .unwrap();
    window.limit_update_rate(Some(std::time::Duration::from_secs_f32(1.0 / 60.0)));

    let mut cpu = Rc::new(Cell::new(CPU::new()));
    let mut mmu = Rc::new(Cell::new(MMU::new()));
    let mut rtc = Rc::new(Cell::new(RTC::new()));
    let mut gpu = Rc::new(Cell::new(GPU::new()));
    let mut joypad = Rc::new(Cell::new(JoyPad::new()));
    let mut ctx = Context {
        cpu,
        mmu,
        rtc,
        gpu,
        joypad,
    };
    // call mmu.load_rom(path) on ctx
    ctx.mmu.get_mut().load_rom(path);

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
                ctx.joypad.get_mut().on_key_pressed(&mut ctx, *to);
            } else if window.is_key_released(*from) {
                ctx.joypad.get_mut().on_key_released(*to);
            }
        }

        let ticks = elapsed * 4194304 / 1000000;
        let mut cycles = 0;
        while cycles < ticks {
            let c = ctx.cpu.get_mut().execute_next_opcode(&mut ctx);
            ctx.rtc.get_mut().update_timers(&mut ctx, c);
            ctx.gpu.get_mut().update_graphics(&mut ctx, c);
            ctx.cpu.get_mut().do_interrupts(&mut ctx);
            cycles += c as u128;
        }

        window
            .update_with_buffer(
                &ctx.gpu.video_buffer,
                constants::SCREEN_WIDTH,
                constants::SCREEN_HEIGHT,
            )
            .unwrap();
    }
}

fn main() {
    run_rom("./roms/pokemon.gb");
}
