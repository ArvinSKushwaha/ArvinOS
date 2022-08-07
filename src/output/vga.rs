#![allow(dead_code)]

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Color {
    Black = 0,
    Blue = 1,
    Green = 2,
    Cyan = 3,
    Red = 4,
    Magenta = 5,
    Brown = 6,
    LightGray = 7,
    DarkGray = 8,
    LightBlue = 9,
    LightGreen = 10,
    LightCyan = 11,
    LightRed = 12,
    Pink = 13,
    Yellow = 14,
    White = 15,
}

#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VGAColor(u8);

impl VGAColor {
    const fn from_fg_bg(fg: Color, bg: Color) -> Self {
        Self((fg as u8) | ((bg as u8) << 4))
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VGAEntry {
    char: u8,
    color: VGAColor,
}

pub static DEFAULT_COLOR: VGAColor = VGAColor::from_fg_bg(Color::White, Color::Black);
pub static EMPTY_ENTRY: VGAEntry = VGAEntry {
    char: b' ',
    color: DEFAULT_COLOR,
};
