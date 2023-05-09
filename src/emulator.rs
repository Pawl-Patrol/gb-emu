use std::io::Result;

use crate::cartridge::{load_rom, load_state, save_state};
use crate::cpu::CPU;
use crate::gpu::{SCREEN_HEIGHT, SCREEN_WIDTH};
use crate::joypad;
use mini_gl_fb::glutin::dpi::LogicalSize;
use mini_gl_fb::glutin::event::VirtualKeyCode as Key;
use mini_gl_fb::glutin::event_loop::EventLoop;
use mini_gl_fb::{get_fancy, ConfigBuilder};

pub struct Emulator {
    cpu: CPU,
    rom_path: String,
    ram_path: String,
    speed: u128,
}

impl Emulator {
    pub fn new(rom_path: &str, ram_path: &str) -> Emulator {
        Emulator {
            cpu: CPU::new(),
            rom_path: rom_path.to_string(),
            ram_path: ram_path.to_string(),
            speed: 100,
        }
    }

    pub fn run(&mut self) {
        self.load_rom()
            .unwrap_or_else(|e| println!("Failed to load rom: {}", e));
        self.load_save().unwrap_or_default();

        let mut event_loop = EventLoop::new();
        let config = ConfigBuilder::default()
            .window_title("Gameboy Emulator".to_string())
            .buffer_size(Some(LogicalSize::new(
                SCREEN_WIDTH as u32,
                SCREEN_HEIGHT as u32,
            )))
            .resizable(true)
            .invert_y(false)
            .build();
        let mut window = get_fancy(config, &event_loop);

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
        let mut last_speed_change = std::time::Instant::now();

        window.glutin_handle_basic_input(&mut event_loop, |fb, input| {
            let now = std::time::Instant::now();
            let elapsed = now.duration_since(previous);
            previous = now;

            if input.key_is_down(Key::Escape) {
                return false;
            } else if input.key_pressed(Key::S) {
                self.save_state()
                    .unwrap_or_else(|e| println!("Failed to save state: {}", e));
            } else if input.key_pressed(Key::L) {
                self.load_save()
                    .unwrap_or_else(|e| println!("Failed to load state: {}", e));
            } else if input.key_is_down(Key::Comma) {
                if self.speed < 1000 && now.duration_since(last_speed_change).as_millis() > 100 {
                    self.speed += 10;
                    last_speed_change = now;
                    println!("Speed: {}%", self.speed)
                }
            } else if input.key_is_down(Key::Period) {
                if self.speed > 10 && now.duration_since(last_speed_change).as_millis() > 100 {
                    self.speed -= 10;
                    last_speed_change = now;
                    println!("Speed: {}%", self.speed)
                }
            }

            for (from, to) in &key_mapping {
                if input.key_pressed(*from) {
                    self.cpu.mmu.interrupt_flag |= self.cpu.mmu.joypad.on_key_pressed(*to);
                } else if input.key_released(*from) {
                    self.cpu.mmu.joypad.on_key_released(*to);
                }
            }

            let ticks = elapsed.as_micros() * 4194304 / 1000000 * self.speed / 100;
            let mut cycles = 0;

            while cycles < ticks {
                cycles += self.cpu.update() as u128;
            }

            fb.update_buffer(&self.cpu.mmu.gpu.video_buffer);

            true
        })
    }

    pub fn load_rom(&mut self) -> Result<()> {
        let rom = load_rom(&self.rom_path)?;
        self.cpu.mmu.cartrige = Some(rom);
        Ok(())
    }

    pub fn load_save(&mut self) -> Result<()> {
        load_state(self.cpu.mmu.cartrige.as_mut().unwrap(), &self.ram_path)
    }

    pub fn save_state(&mut self) -> Result<()> {
        save_state(self.cpu.mmu.cartrige.as_ref().unwrap(), &self.ram_path)
    }
}
