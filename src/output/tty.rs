use core::fmt::{Arguments, Write};

use spin::Mutex;
use voladdress::{Safe, VolBlock};

use super::vga::{VGAEntry, DEFAULT_COLOR, EMPTY_ENTRY, VGA_HEIGHT, VGA_WIDTH};

const LAST_ROW_OFFSET: usize = VGA_WIDTH * (VGA_HEIGHT - 1);

/// The object through which we may write to the VGA buffer. This must be wrapped with
/// a lock of some form to allow for static usage. This is done with the [`SpinMutex`] type.
struct Writer {
    buffer: VolBlock<VGAEntry, Safe, Safe, { VGA_WIDTH * VGA_HEIGHT }>,
    column: usize,
}

impl Writer {
    const unsafe fn new(addr: usize) -> Self {
        Self {
            buffer: VolBlock::new(addr),
            column: 0,
        }
    }

    fn add_line(&mut self) {
        self.column = 0;
        for i in 0..VGA_WIDTH * (VGA_HEIGHT - 1) {
            self.buffer
                .index(i)
                .write(self.buffer.index(i + VGA_HEIGHT).read());
        }

        for i in 0..VGA_WIDTH {
            self.buffer.index(i + LAST_ROW_OFFSET).write(EMPTY_ENTRY);
        }
    }

    fn add_byte(&mut self, c: u8) {
        match c {
            b'\n' => self.add_line(),
            0x20..=0x7e => {
                if self.column == VGA_WIDTH {
                    self.add_line();
                }

                self.buffer
                    .index(self.column + LAST_ROW_OFFSET)
                    .write(VGAEntry {
                        char: c as u8,
                        color: DEFAULT_COLOR,
                    });

                self.column += 1;
            }
            _ => self.add_byte(b'?'),
        }
    }
}

impl Write for Writer {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        s.bytes().for_each(|c| self.add_byte(c));
        Ok(())
    }
}

lazy_static::lazy_static! {
    static ref WRITER: Mutex<Writer> = Mutex::new(unsafe { Writer::new(0xb8000) });
}

#[doc(hidden)]
pub fn _print(args: Arguments) {
    WRITER.lock().write_fmt(args).unwrap();
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::output::tty::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => (print!("\n"));
    ($($arg:tt)*) => (print!("{}\n", format_args!($($arg)*)));
}
