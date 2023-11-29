use spin::Mutex;
use voladdress::{Safe, VolGrid2d};

const EGA_WIDTH: usize = 80;
const EGA_HEIGHT: usize = 25;
const EGA_LOCATION: usize = 0xB8000;

#[repr(u8)]
#[derive(Copy, Clone, Debug)]
enum CgaBaseColor {
    Black = 0,
    Blue = 1,
    Green = 2,
    Cyan = 3,
    Red = 4,
    Magenta = 5,
    Brown = 6,
    White = 7,
    LightBlack = 8,
    LightBlue = 9,
    LightGreen = 10,
    LightCyan = 11,
    LightRed = 12,
    LightMagenta = 13,
    LightBrown = 14,
    LightWhite = 15,
}

#[repr(transparent)]
#[derive(Copy, Clone, Debug)]
struct CgaColor(u8);

impl CgaColor {
    pub const fn new(fg: CgaBaseColor, bg: CgaBaseColor) -> CgaColor {
        CgaColor((fg as u8) | (bg as u8) << 4)
    }

    pub const fn color(&self, byte: u8) -> CgaCode {
        CgaCode((self.0 as u16) << 8 | (byte as u16))
    }
}

#[repr(transparent)]
#[derive(Copy, Clone, Debug)]
struct CgaCode(u16);

struct Buffer(VolGrid2d<CgaCode, Safe, Safe, EGA_WIDTH, EGA_HEIGHT>);

impl Buffer {
    pub const fn size(&self) -> (usize, usize) {
        (EGA_HEIGHT, EGA_WIDTH)
    }
}

struct EgaWriterInner {
    buffer: Buffer,
    cursor: (usize, usize),
    current_color: CgaColor,
}

#[derive(Copy, Clone, Debug)]
pub enum EgaWriteError {
    OutOfBoundsWrite,
    NonAsciiCharacter,
}

pub struct EgaWriter(Mutex<EgaWriterInner>);

pub static WRITER: EgaWriter = EgaWriter(Mutex::new(
    EgaWriterInner {
        /// # SAFETY
        /// This operation is safe as this memory location is a valid location to write (shared
        /// memory), and is aligned to a 16-bit (2-byte) boundary. The bounds of the [`VolGrid2d`]
        /// are also correct, as `25x80` is the default EGA text mode layout.
        buffer: Buffer(unsafe { VolGrid2d::new(EGA_LOCATION) }),
        cursor: (0, 0),
        current_color: CgaColor::new(CgaBaseColor::White, CgaBaseColor::Black),
    }
));

impl EgaWriterInner {
    pub fn set_color(&mut self, color: CgaColor) {
        self.current_color = color;
    }

    pub fn write_byte_at(&mut self, byte: u8, pos: (usize, usize)) -> Result<(), EgaWriteError> {
        if let Some(mem) = self.buffer.0.get(pos.0, pos.1) {
            mem.write(self.current_color.color(byte));

            Ok(())
        } else {
            Err(EgaWriteError::OutOfBoundsWrite)
        }
    }

    pub fn write_char(&mut self, char: char) -> Result<(), EgaWriteError> {
        if char.is_ascii() {
            self.write_byte_at(char as u8, self.cursor)?;
            self.increment_cursor();

            Ok(())
        } else {
            Err(EgaWriteError::NonAsciiCharacter)
        }
    }

    pub fn increment_cursor(&mut self) {
        let (y, x) = self.cursor;
        let (h, w) = self.buffer.size();

        if x == usize::MAX || x + 1 == w {
            self.cursor.1 = 0;

            if y == usize::MAX || y + 1 == h {
                self.move_lines_up();
            } else {
                self.cursor.0 += 1;
            }
        } else {
            self.cursor.1 += 1;
        }
    }

    fn move_lines_up(&mut self) {
        for row in 0..(self.buffer.size().0 - 1) {
            for col in 0..self.buffer.size().1 {
                self.buffer
                    .0
                    .index(col, row)
                    .write(self.buffer.0.index(col, row + 1).read());
            }
        }

        for col in 0..self.buffer.size().1 {
            self.buffer
                .0
                .index(col, self.buffer.size().0 - 1)
                .write(self.current_color.color(b' '));
        }
    }

    pub fn write_str(&mut self, str: &str) -> Result<(), EgaWriteError> {
        for c in str.chars() {
            self.write_char(c)?;
        }

        Ok(())
    }
}

impl EgaWriter {
    pub fn write_str(&self, str: &str) -> Result<(), EgaWriteError> {
        self.0.lock().write_str(str)
    }

    pub fn write_char(&self, char: char) -> Result<(), EgaWriteError> {
        self.0.lock().write_char(char)
    }
}
