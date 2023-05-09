mod canvas;
mod cartridge;
mod constants;
mod cpu;
mod gpu;
mod joypad;
mod mmu;
mod rtc;
mod traits;

use std::{
    fs::{File, OpenOptions},
    io::Write,
};

use cpu::CPU;
use minifb::Window;

fn run_rom(path: &str, log: bool) {
    let mut window = Window::new(
        "Gameboy Emulator",
        constants::SCREEN_WIDTH,
        constants::SCREEN_HEIGHT,
        minifb::WindowOptions::default(),
    )
    .unwrap();
    window.limit_update_rate(Some(std::time::Duration::from_secs_f32(1.0 / 60.0)));

    let mut cpu = CPU::new();
    cpu.mmu.load_rom(path);

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

    let mut file: Option<File> = None;
    if log {
        // delete old file if exists
        std::fs::remove_file("out.txt").unwrap_or_default();
        // create new empty file in append mode
        file = OpenOptions::new()
            .write(true)
            .create(true)
            .append(true)
            .open("out.txt")
            .ok();
    }

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
                cpu.mmu.joypad.on_key_pressed(*to);
            } else if window.is_key_released(*from) {
                cpu.mmu.joypad.on_key_released(*to);
            }
        }

        if let Some(i) = cpu.mmu.joypad.needs_interrupt {
            cpu.request_interrupt(i);
            cpu.mmu.joypad.needs_interrupt = None;
        }

        let mut ticks = elapsed * 4194304 / 1000000;
        if ticks > 69905 {
            // println!("too slow: {}", ticks);
            ticks = 69905;
        }
        let mut cycles = 0;
        while cycles < 69905 {
            // // log A:00 F:11 B:22 C:33 D:44 E:55 H:66 L:77 SP:8888 PC:9999 PCMEM:AA,BB,CC,DD
            // if let Some(f) = &mut file {
            //     let str  = format!("A:{:02X} F:{:02X} B:{:02X} C:{:02X} D:{:02X} E:{:02X} H:{:02X} L:{:02X} SP:{:04X} PC:{:04X} PCMEM:{:02X},{:02X},{:02X},{:02X}\n",
            //     cpu.a,
            //     cpu.f,
            //     cpu.b,
            //     cpu.c,
            //     cpu.d,
            //     cpu.e,
            //     cpu.h,
            //     cpu.l,
            //     cpu.sp,
            //     cpu.pc,
            //     cpu.mmu.read(cpu.pc),
            //     cpu.mmu.read(cpu.pc + 1),
            //     cpu.mmu.read(cpu.pc + 2),
            //     cpu.mmu.read(cpu.pc + 3),
            // );
            //     f.write_all(str.as_bytes()).unwrap();
            // }
            let op_cycles = cpu.execute_next_opcode();
            cpu.mmu.rtc.update_timers(op_cycles);
            if let Some(i) = cpu.mmu.rtc.needs_interrupt {
                cpu.request_interrupt(i);
                cpu.mmu.rtc.needs_interrupt = None;
            }
            cpu.mmu.gpu.update_graphics(op_cycles);
            if let Some(i) = cpu.mmu.gpu.needs_interrupt {
                cpu.request_interrupt(i);
                cpu.mmu.gpu.needs_interrupt = None;
            }
            cpu.do_interrupts();
            cycles += op_cycles as u128;
        }

        window
            .update_with_buffer(
                &cpu.mmu.gpu.video_buffer,
                constants::SCREEN_WIDTH,
                constants::SCREEN_HEIGHT,
            )
            .unwrap();
    }
}

fn main() {
    // run_rom("./roms/01-special.gb", true);
    // run_rom("./roms/02-interrupts.gb", true);
    // run_rom("./roms/03-op sp,hl.gb", true);
    // run_rom("./roms/04-op r,imm.gb", true);
    // run_rom("./roms/05-op rp.gb", true);
    // run_rom("./roms/06-ld r,r.gb", true);
    // run_rom("./roms/07-jr,jp,call,ret,rst.gb", true);
    // run_rom("./roms/08-misc instrs.gb", true);
    // run_rom("./roms/09-op r,r.gb", true);
    // run_rom("./roms/10-bit ops.gb", true);
    // run_rom("./roms/11-op a,(hl).gb", true);
    // run_rom("./roms/tetris.gb", true);
    // run_rom(
    //     "./tests/mooneye-test-suite/acceptance/add_sp_e_timing.gb",
    //     false,
    // );
    // run_rom("./tests/age-test-roms/halt/ei-halt-dmgC-cgbBCE.gb", false);
    // run_rom("./tests/bully/bully.gb", true);
    run_rom("./roms/pikachu.gb", false);
}
