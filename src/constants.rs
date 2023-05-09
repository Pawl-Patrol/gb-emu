pub const CPU_FREQ_IN_HZ: u32 = 4194304;
pub const FRAME_RATE: u32 = 60;
pub const CYCLES_PER_FRAME: u32 = CPU_FREQ_IN_HZ / FRAME_RATE;

// timer
pub const TIMER_FREQ_0: u32 = CPU_FREQ_IN_HZ / 4096;
pub const TIMER_FREQ_1: u32 = CPU_FREQ_IN_HZ / 262144;
pub const TIMER_FREQ_2: u32 = CPU_FREQ_IN_HZ / 65536;
pub const TIMER_FREQ_3: u32 = CPU_FREQ_IN_HZ / 16384;

// color palette
pub type Color = u8;
pub const COLOR_WHITE: Color = 0b00;
pub const COLOR_LIGHT_GRAY: Color = 0b01;
pub const COLOR_DARK_GRAY: Color = 0b10;
pub const COLOR_BLACK: Color = 11;

// screen size
pub const SCREEN_WIDTH: usize = 160;
pub const SCREEN_HEIGHT: usize = 144;

// bank modes
pub const MBC1: u8 = 0x01;
pub const MBC2: u8 = 0x02;

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
pub const SCANLINE_CYCLES: u32 = 456;

// arithmetic
pub const FLAG_ZERO: u8 = 7;
pub const FLAG_SUBTRACT: u8 = 6;
pub const FLAG_HALF_CARRY: u8 = 5;
pub const FLAG_CARRY: u8 = 4;
