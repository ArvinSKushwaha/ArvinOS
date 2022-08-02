use core::fmt::Write;

use spin::mutex::SpinMutex;
use voladdress::{Safe, VolAddress, VolBlock};

mod vga;

use vga::{VGAEntry, VGAEntryColor, DEFAULT_VGA_COLOR, VGA_HEIGHT, VGA_WIDTH};

static WRITER: Writer = Writer(SpinMutex::new(WriterInner {
    cursor: (0, 0),
    // SAFETY: We know that our VGA buffer is located as 0xb8000
    buffer: Buffer(unsafe { VolBlock::new(0xb8000) }),
    current_color: DEFAULT_VGA_COLOR,
}));

struct Buffer(VolBlock<VGAEntry, Safe, Safe, { VGA_WIDTH * VGA_HEIGHT }>);

impl Buffer {
    fn index(&self, idx: (usize, usize)) -> VolAddress<VGAEntry, Safe, Safe> {
        self.0.index(idx.0 * VGA_WIDTH + idx.1)
    }
}

pub struct Writer(SpinMutex<WriterInner>);
unsafe impl Sync for Writer {}

struct WriterInner {
    cursor: (usize, usize),
    buffer: Buffer,
    current_color: VGAEntryColor,
}

impl WriterInner {
    fn new_line(&mut self) {
        if self.cursor.0 != VGA_HEIGHT - 1 {
            self.cursor.0 += 1;
        } else {
            for i in 0..(VGA_HEIGHT - 1) {
                for j in 0..VGA_WIDTH {
                    self.buffer
                        .index((i, j))
                        .write(self.buffer.index((i + 1, j)).read());
                }
            }

            for j in 0..VGA_WIDTH {
                self.buffer.index((VGA_HEIGHT - 1, j)).write(VGAEntry {
                    byte: b' ',
                    color: DEFAULT_VGA_COLOR,
                });
            }
        }
    }

    fn carriage_return(&mut self) {
        self.cursor.1 = 0;
    }

    pub fn add_char(&mut self, char: char) {
        if char == '\n' {
            self.new_line();
            self.carriage_return();
        } else if char == '\r' {
            self.carriage_return();
        } else if char.is_ascii() {
            self.buffer.index(self.cursor).write(VGAEntry {
                byte: char as u8,
                color: self.current_color,
            });
            self.cursor.1 += 1;
            if self.cursor.1 == VGA_WIDTH {
                self.new_line();
                self.carriage_return();
            }
        } else {
            self.add_char('?');
        }
    }

    pub fn add_str(&mut self, str: &str) {
        str.chars().for_each(|c| self.add_char(c))
    }
}

impl Write for WriterInner {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        self.add_str(s);
        Ok(())
    }

    fn write_char(&mut self, c: char) -> core::fmt::Result {
        self.add_char(c);
        Ok(())
    }
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {
        $crate::output::_print(format_args!($($arg)*))
    };
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => {
        $crate::output::tty::_print(format_args!($($arg)*));
        $crate::output::tty::_print(format_args!("\n"));
    };
}

#[doc(hidden)]
pub fn _print(args: core::fmt::Arguments) {
    let mut lock = WRITER.0.lock();
    lock.write_fmt(args).ok();
}
