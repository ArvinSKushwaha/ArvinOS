use core::fmt::Write;

use spin::mutex::SpinMutex;
use voladdress::{Safe, VolAddress};

mod vga;

use vga::{VGAEntry, VGAEntryColor, DEFAULT_VGA_COLOR};

pub(super) static WRITER: Writer = Writer(SpinMutex::new(None));

unsafe impl Sync for Writer {}

struct Buffer {
    data: *mut VGAEntry,
    width: usize,
    height: usize,
}

impl Buffer {
    const unsafe fn new(addr: usize, width: usize, height: usize) -> Self {
        Buffer {
            data: addr as *mut VGAEntry,
            width,
            height,
        }
    }
    fn index(&self, idx: (usize, usize)) -> VolAddress<VGAEntry, Safe, Safe> {
        assert!(idx.0 < self.width && idx.1 < self.height);
        unsafe { VolAddress::new(self.data.add(idx.0 * self.width + idx.1) as usize) }
    }
}

pub struct Writer(SpinMutex<Option<WriterInner>>);

impl Writer {
    pub(super) fn initialize(
        &self,
        buffer_addr: usize,
        width: usize,
        height: usize,
    ) -> Result<(), ()> {
        let mut lock = self.0.lock();
        if lock.is_some() {
            return Err(());
        } else {
            lock.replace(WriterInner {
                cursor: (0, 0),
                width,
                height,
                buffer: unsafe { Buffer::new(buffer_addr, width, height) },
                current_color: DEFAULT_VGA_COLOR,
            });
        }
        Ok(())
    }
}

struct WriterInner {
    cursor: (usize, usize),
    width: usize,
    height: usize,
    buffer: Buffer,
    current_color: VGAEntryColor,
}

impl WriterInner {
    fn new_line(&mut self) {
        if self.cursor.0 != self.height - 1 {
            self.cursor.0 += 1;
        } else {
            for i in 0..(self.height - 1) {
                for j in 0..self.width {
                    self.buffer
                        .index((i, j))
                        .write(self.buffer.index((i + 1, j)).read());
                }
            }

            for j in 0..self.width {
                self.buffer.index((self.height - 1, j)).write(VGAEntry {
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
            if self.cursor.1 == self.width {
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
    // Effectively waits for the lock to be initialized.
    loop {
        let mut lock = WRITER.0.lock();
        if lock.is_some() {
            lock.as_mut().unwrap().write_fmt(args).ok();
            break;
        }
    }
}
