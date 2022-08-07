use core::{
    cmp::Ordering,
    fmt::{Arguments, Write},
    num::NonZeroUsize,
};

use spin::{lazy::Lazy, mutex::SpinMutex};
use voladdress::{Safe, VolAddress};

use super::vga::{VGAEntry, DEFAULT_COLOR, EMPTY_ENTRY};

/// A location within a VGA Buffer.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
struct Cursor {
    x: usize,
    y: usize,
}

/// The buffer containing [`VGAEntry`] elements as well as the
/// dimensions of the buffer.
struct Buffer {
    data: VolAddress<VGAEntry, Safe, Safe>,
    width: usize,
    height: usize,
}

impl Buffer {
    /// Create a new Buffer with given dimensions and address.
    ///
    /// # Safety
    ///
    /// - Never create two buffer such that the address ranges of the
    /// buffers overlap.
    /// - The address must be aligned to `core::mem::align_of::<VGAEntry>()`.
    /// - `width * height` must not exceed `isize::MAX`. Also, see [`core::ptr::offset`].
    /// - No other accesses, read or write, are permitted to any addresses to be
    /// contained in this buffer: `addr..(addr + width * height * mem::size_of::<VGAEntry>())`, for the lifetime
    /// of the buffer.
    ///
    unsafe fn new(addr: usize, width: NonZeroUsize, height: NonZeroUsize) -> Self {
        // Ensure all data is initialized
        let ptr = VolAddress::<VGAEntry, Safe, Safe>::new(addr);

        for i in 0..(width.get() * height.get()) {
            ptr.add(i).write(EMPTY_ENTRY);
        }

        Self {
            data: ptr,
            width: width.get(),
            height: height.get(),
        }
    }

    /// Write [`VGAEntry`] to location represented by the cursor (only
    /// if location in cursor is within bounds). An attempted out of bounds
    /// write is not executed.
    ///
    /// The method will return `false` on an out-of-bounds write attempt or
    /// if the given cursor is invalid. Otherwise, it will perform the
    /// write and return `true`.
    fn write(&mut self, cursor: Cursor, data: VGAEntry) -> bool {
        // SAFETY: The creation of this buffer requires that the width
        // and height are given correctly in [`Buffer::new`]. Writes
        // out-of-bounds are ignored by the `if let` in the code. This
        // function also requires an exclusive reference to this data.
        // It is therefore known that this data cannot be mutated from
        // multiple threads.

        // ^ The above is just a massive fuck-you to myself if I misthought this
        // lmao
        if let Some(ptr) = self.ptr_at(cursor) {
            ptr.write(data);
            true
        } else {
            false
        }
    }

    /// Read [`VGAEntry`] from location represented by the cursor (only
    /// if location in cursor is within bounds). An attempted out of bounds
    /// read is not executed.
    ///
    /// If the cursor is out-of-bounds, [`None`] is returned. Otherwise,
    /// [`Some(VGAEntry)`] is returned.
    fn read(&self, cursor: Cursor) -> Option<VGAEntry> {
        // SAFETY: Like with [`Buffer::write`], the bounds are assumed to
        // be given correctly when the unsafe [`Buffer::new`] method is
        // called. Because race conditions create undefined behavior,
        // a mutable reference to the buffer is required to write,
        // so reads may not be race with writes.
        self.ptr_at(cursor).map(|ptr| ptr.read())
    }

    // #[inline(always)]
    fn ptr_at(&self, cursor: Cursor) -> Option<VolAddress<VGAEntry, Safe, Safe>> {
        if cursor.x < self.width && cursor.y < self.height {
            Some(unsafe { self.data.add(cursor.y * self.width + cursor.x) })
        } else {
            panic!("Yeet");
            None
        }
    }
}

/// The object through which we may write to the Buffer. This must be wrapped with
/// a lock of some form to allow for static usage. This is done with the [`Writer`] type.
struct WriterInner {
    buffer: Buffer,
    cursor: Cursor,
}

impl WriterInner {
    fn new(buffer: Buffer) -> Self {
        Self {
            buffer,
            cursor: Cursor { x: 0, y: 0 },
        }
    }

    fn increment_position(&mut self) {
        // `width * height` must be able to fit in `isize` by the safety rules for Buffer.
        // They also must be greater than 0. Therefore, checking if the `cursor < limit - 1`
        // will work fine.
        match self.cursor.x.cmp(&(self.buffer.width - 1)) {
            Ordering::Less => self.cursor.x += 1,
            Ordering::Equal => self.add_line(),
            _ => {}
        }
    }

    fn add_line(&mut self) {
        self.cursor.x = 0;

        match self.cursor.y.cmp(&(self.buffer.height - 1)) {
            Ordering::Less => self.cursor.y += 1,
            Ordering::Equal => {
                for i in 1..self.buffer.height {
                    for j in 0..self.buffer.width {
                        self.buffer.write(
                            Cursor { x: j, y: i - 1 },
                            self.buffer.read(Cursor { x: j, y: i }).unwrap(),
                        );
                    }
                }

                for j in 0..self.buffer.width {
                    self.buffer.write(
                        Cursor {
                            x: j,
                            y: self.buffer.height - 1,
                        },
                        EMPTY_ENTRY,
                    );
                }
            }
            _ => {}
        }

        // if self.cursor.y < self.buffer.height - 1 {
        //     // If we haven't filled the buffer yet, just keep going
        //     self.cursor.y += 1;
        // } else if self.cursor.y == self.buffer.height {
        //     // If there's no more lines in the buffer, shift everything up one line.
        //     for i in 1..self.buffer.height {
        //         for j in 0..self.buffer.width {
        //             self.buffer.write(
        //                 Cursor { x: j, y: i - 1 },
        //                 self.buffer.read(Cursor { x: j, y: i }).unwrap(),
        //             );
        //         }
        //     }
        //
        //     for j in 0..self.buffer.width {
        //         self.buffer.write(
        //             Cursor {
        //                 x: j,
        //                 y: self.buffer.height - 1,
        //             },
        //             EMPTY_ENTRY,
        //         );
        //     }
        // }
    }

    fn add_byte(&mut self, c: u8) {
        match c {
            b'\n' => self.add_line(),
            0x20..=0x7e => {
                self.buffer.write(
                    self.cursor,
                    VGAEntry {
                        char: c as u8,
                        color: DEFAULT_COLOR,
                    },
                );
                self.increment_position();
            }
            _ => self.add_byte(b'?'),
        }
    }
}

impl Write for WriterInner {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        s.bytes().for_each(|c| self.add_byte(c));
        Ok(())
    }
}

struct Writer(SpinMutex<WriterInner>);

// These are safe as [`Writer`] will keeps the [`*mut VGAEntry`]
// behind safe functions in [`Buffer`] with proper aliasing rules.
unsafe impl Send for Writer {}
unsafe impl Sync for Writer {}

static WRITER: Lazy<Writer> = Lazy::new(|| {
    Writer(SpinMutex::new(WriterInner::new(unsafe {
        Buffer::new(
            0xb8000,
            NonZeroUsize::new(80).unwrap(),
            NonZeroUsize::new(25).unwrap(),
        )
    })))
});

#[doc(hidden)]
pub fn _print(args: Arguments) {
    WRITER.0.lock().write_fmt(args).ok();
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
