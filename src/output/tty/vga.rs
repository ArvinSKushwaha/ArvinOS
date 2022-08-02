#[allow(dead_code)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum VGAColor {
    Black = 0,
    Blue = 1,
    Green = 2,
    Cyan = 3,
    Red = 4,
    Magenta = 5,
    Brown = 6,
    LightGrey = 7,
    DarkGrey = 8,
    LightBlue = 9,
    LightGreen = 10,
    LightCyan = 11,
    LightRed = 12,
    LightMagenta = 13,
    LightBrown = 14,
    White = 15,
}

#[repr(transparent)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct VGAEntryColor(u8);

impl VGAEntryColor {
    const fn from_fg_bg(fg: VGAColor, bg: VGAColor) -> Self {
        Self(((bg as u8) << 4) | (fg as u8))
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct VGAEntry {
    pub byte: u8,
    pub color: VGAEntryColor,
}

pub const VGA_HEIGHT: usize = 25;
pub const VGA_WIDTH: usize = 80;
pub const DEFAULT_VGA_COLOR: VGAEntryColor =
    VGAEntryColor::from_fg_bg(VGAColor::White, VGAColor::Black);
