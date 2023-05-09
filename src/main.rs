mod cartridge;
mod cpu;
mod gpu;
mod joypad;
mod mmu;
mod rtc;
mod traits;

fn run_rom(path: &str) {
    let mut window = minifb::Window::new(
        "Gameboy Emulator",
        gpu::SCREEN_WIDTH,
        gpu::SCREEN_HEIGHT,
        minifb::WindowOptions::default(),
    )
    .unwrap();
    window.limit_update_rate(Some(std::time::Duration::from_secs_f32(1.0 / 60.0)));

    let mut cpu = cpu::CPU::new();
    cpu.mmu.load_rom(path);

    let key_mapping = vec![
        (minifb::Key::Up, joypad::KEY_UP),
        (minifb::Key::Down, joypad::KEY_DOWN),
        (minifb::Key::Left, joypad::KEY_LEFT),
        (minifb::Key::Right, joypad::KEY_RIGHT),
        (minifb::Key::A, joypad::KEY_A),
        (minifb::Key::B, joypad::KEY_B),
        (minifb::Key::Enter, joypad::KEY_START),
        (minifb::Key::Space, joypad::KEY_SELECT),
    ];

    // let mut file: Option<std::fs::File> = None;
    // if log {
    //     // delete old file if exists
    //     std::fs::remove_file("out.txt").unwrap_or_default();
    //     // create new empty file in append mode
    //     file = std::fs::OpenOptions::new()
    //         .write(true)
    //         .create(true)
    //         .append(true)
    //         .open("out.txt")
    //         .ok();
    // }

    let mut last = std::time::Instant::now();
    loop {
        let now = std::time::Instant::now();
        let elapsed = now.duration_since(last).as_micros();
        last = now;

        let t1 = std::time::Instant::now();
        if window.is_key_down(minifb::Key::Escape) {
            std::process::exit(0);
        }
        for (from, to) in &key_mapping {
            if window.is_key_down(*from) {
                cpu.mmu.interrupt_flag |= cpu.mmu.joypad.on_key_pressed(*to);
            } else if window.is_key_released(*from) {
                cpu.mmu.joypad.on_key_released(*to);
            }
        }
        let t2 = std::time::Instant::now();

        let mut ticks = elapsed * 4194304 / 1000000;

        if ticks > 69905 {
            ticks = 69905;
        }

        let mut cycles = 0;
        while cycles < ticks {
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
            cycles += cpu.update() as u128;
        }
        let t3 = std::time::Instant::now();

        window
            .update_with_buffer(
                &cpu.mmu.gpu.video_buffer.clone(),
                gpu::SCREEN_WIDTH,
                gpu::SCREEN_HEIGHT,
            )
            .unwrap();
        let t4 = std::time::Instant::now();

        println!(
            "joypad: {}ms, cpu: {}ms, window: {}ms",
            t2.duration_since(t1).as_millis(),
            t3.duration_since(t2).as_millis(),
            t4.duration_since(t3).as_millis()
        );
    }
}

fn main() {
    // run_rom("./roms/01-special.gb");
    // run_rom("./roms/02-interrupts.gb");
    // run_rom("./roms/03-op sp,hl.gb");
    // run_rom("./roms/04-op r,imm.gb");
    // run_rom("./roms/05-op rp.gb");
    // run_rom("./roms/06-ld r,r.gb");
    // run_rom("./roms/07-jr,jp,call,ret,rst.gb");
    // run_rom("./roms/08-misc instrs.gb");
    // run_rom("./roms/09-op r,r.gb");
    // run_rom("./roms/10-bit ops.gb");
    // run_rom("./roms/11-op a,(hl).gb");
    // run_rom("./roms/tetris.gb");
    // run_rom("./tests/mooneye-test-suite/acceptance/add_sp_e_timing.gb");
    // run_rom("./tests/age-test-roms/halt/ei-halt-dmgC-cgbBCE.gb");
    // run_rom("./tests/bully/bully.gb");
    run_rom("./roms/pikachu.gb");
}
