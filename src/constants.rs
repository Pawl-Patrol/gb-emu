pub const CPU_FREQ_IN_HZ: u32 = 4194304;
pub const FRAME_RATE: u32 = 60;
pub const CYCLES_PER_FRAME: u32 = 69905;

// timer
pub const TIMER_FREQ_0: u32 = 1024;
pub const TIMER_FREQ_1: u32 = 16;
pub const TIMER_FREQ_2: u32 = 64;
pub const TIMER_FREQ_3: u32 = 256;

// screen size
pub const SCREEN_WIDTH: usize = 160;
pub const SCREEN_HEIGHT: usize = 144;

// bank sizes
pub const ROM_BANK_SIZE: usize = 0x4000;
pub const RAM_BANK_SIZE: usize = 0x2000;

// special addresses
pub const TIMA: u16 = 0xFF05;
pub const TMA: u16 = 0xFF06;
pub const TMC: u16 = 0xFF07;
pub const DIV: u16 = 0xFF04;
pub const INTERRUPT_FLAG: u16 = 0xFF0F;
pub const INTERRUPT_ENABLE: u16 = 0xFFFF;
pub const LYC: u16 = 0xFF45;
pub const DMA: u16 = 0xFF46;
pub const LCD_CONTROL: u16 = 0xFF40;
pub const LCD_STATUS: u16 = 0xFF41;
pub const SCANLINE: u16 = 0xFF44;

// graphics
pub const SCANLINE_CYCLES: u16 = 456;

// arithmetic
pub const FLAG_ZERO: u8 = 7;
pub const FLAG_SUBTRACT: u8 = 6;
pub const FLAG_HALF_CARRY: u8 = 5;
pub const FLAG_CARRY: u8 = 4;

// key maps
pub const KEY_UP: u8 = 2;
pub const KEY_DOWN: u8 = 3;
pub const KEY_LEFT: u8 = 1;
pub const KEY_RIGHT: u8 = 0;
pub const KEY_A: u8 = 4;
pub const KEY_B: u8 = 5;
pub const KEY_START: u8 = 7;
pub const KEY_SELECT: u8 = 6;
