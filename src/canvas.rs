macro_rules! rgb {
    ($r:expr, $g:expr, $b:expr) => {
        ($r << 16) | ($g << 8) | $b
    };
}

// color palette
pub const COLOR_WHITE: u32 = rgb!(0xFF, 0xFF, 0xFF);
pub const COLOR_LIGHT_GRAY: u32 = rgb!(0xCC, 0xCC, 0xCC);
pub const COLOR_DARK_GRAY: u32 = rgb!(0x77, 0x77, 0x77);
pub const COLOR_BLACK: u32 = rgb!(0x00, 0x00, 0x00);
