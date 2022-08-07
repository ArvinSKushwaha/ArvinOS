use core::{
    cmp::Ordering,
    fmt::{Arguments, Write},
    num::NonZeroUsize,
};

use spin::{lazy::Lazy, mutex::SpinMutex};

use super::vga::{VGAEntry, DEFAULT_COLOR, EMPTY_ENTRY};

/// A location within a VGA Buffer.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Cursor {
    pub x: usize,
    pub y: usize,
}

/// The buffer containing [`VGAEntry`] elements as well as the
/// dimensions of the buffer.
struct Buffer {
    data: &'static mut [VGAEntry],
    width: usize,
    height: usize,
}

impl Buffer {
    /// Create a new Buffer with given dimensions and address.
    ///
    /// # Safety
    ///
    /// See [`core::slice::from_raw_parts_mut`]. Safety requirements have been copied below
    /// as follows:
    ///
    /// Behavior is undefined if any of the following conditions are violated:
    ///
    /// * `data` must be [valid] for both reads and writes for `len * mem::size_of::<T>()` many bytes,
    ///   and it must be properly aligned. This means in particular:
    ///
    ///     * The entire memory range of this slice must be contained within a single allocated object!
    ///       Slices can never span across multiple allocated objects.
    ///     * `data` must be non-null and aligned even for zero-length slices. One
    ///       reason for this is that enum layout optimizations may rely on references
    ///       (including slices of any length) being aligned and non-null to distinguish
    ///       them from other data. You can obtain a pointer that is usable as `data`
    ///       for zero-length slices using [`NonNull::dangling()`].
    ///
    /// * `data` must point to `len` consecutive properly initialized values of type `T`.
    ///     - This is handled, as all the data within the given range is initialized to
    ///     [`EMPTY_ENTRY`]
    ///
    ///
    /// * The memory referenced by the returned slice must not be accessed through any other pointer
    ///   (not derived from the return value) for the duration of lifetime `'a`.
    ///   Both read and write accesses are forbidden.
    ///
    /// * The total size `len * mem::size_of::<T>()` of the slice must be no larger than `isize::MAX`.
    ///   See the safety documentation of [`pointer::offset`].
    ///
    /// [valid]: ptr#safety
    /// [`NonNull::dangling()`]: ptr::NonNull::dangling
    pub unsafe fn new(addr: usize, width: NonZeroUsize, height: NonZeroUsize) -> Self {
        // Ensure all data is initialized
        let ptr = addr as *mut VGAEntry;

        for i in 0..(width.get() * height.get()) {
            ptr.add(i).write(EMPTY_ENTRY);
        }

        Self {
            data: core::slice::from_raw_parts_mut(ptr, width.get() * height.get()),
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
    pub fn write(&mut self, cursor: Cursor, data: VGAEntry) -> bool {
        // SAFETY: The creation of this buffer requires that the width
        // and height are given correctly in [`Buffer::new`]. Writes
        // out-of-bounds are ignored by the `if let` in the code. This
        // function also requires an exclusive reference to this data.
        // It is therefore known that this data cannot be mutated from
        // multiple threads.

        // ^ The above is just a massive fuck-you to myself if I misthought this
        // lmao
        if let Some(index) = self.compute_index(cursor) {
            unsafe {
                self.data.as_mut_ptr().add(index).write_volatile(data);
                true
            }
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
    pub fn read(&self, cursor: Cursor) -> Option<VGAEntry> {
        // SAFETY: Like with [`Buffer::write`], the bounds are assumed to
        // be given correctly when the unsafe [`Buffer::new`] method is
        // called. Because race conditions create undefined behavior,
        // a mutable reference to the buffer is required to write,
        // so reads may not be race with writes.
        self.compute_index(cursor)
            .map(|index| unsafe { self.data.as_ptr().add(index).read_volatile() })
    }

    // #[inline(always)]
    fn compute_index(&self, cursor: Cursor) -> Option<usize> {
        if cursor.x < self.width && cursor.y < self.height {
            Some(cursor.y * self.width + cursor.x)
        } else {
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
            Ordering::Equal => {
                self.cursor.x = 0;
                self.add_line();
            }
            _ => {}
        }
    }

    fn add_line(&mut self) {
        self.cursor.x = 0;
        if self.cursor.y < self.buffer.height - 1 {
            // If we haven't filled the buffer yet, just keep going
            self.cursor.y += 1;
        } else if self.cursor.y == self.buffer.height {
            // If there's no more lines in the buffer, shift everything up one line.
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

pub struct Writer(SpinMutex<WriterInner>);

static WRITER: Lazy<Writer> = Lazy::new(|| {
    Writer(SpinMutex::new(WriterInner::new(unsafe {
        Buffer::new(
            0xB8000,
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
