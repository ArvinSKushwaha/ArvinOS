use super::vga::VGAEntry;

struct Cursor {
    x: usize,
    y: usize,
}

/// The buffer containing [`VGAEntry`] elements as well as the
/// dimensions of the buffer.
struct Buffer {
    data: *mut VGAEntry,
    width: usize,
    height: usize,
}

impl Buffer {
    /// Write [`VGAEntry`] to location represented by the cursor (only
    /// if location in cursor is within bounds). An attempted out of bounds
    /// write is not executed.
    ///
    /// The method will return `false` on an out-of-bounds write attempt or
    /// if the given cursor is invalid. Otherwise, it will perform the
    /// write and return `true`.
    fn write(&mut self, cursor: Cursor, data: VGAEntry) -> bool {
        // SAFETY: The creation of this buffer requires that the width
        // and height are given correctly. Writes out-of-bounds are ignored
        // by the `if let` in the code. This function also requires an exclusive
        // reference to this data. It is therefore known that this data cannot
        // be mutated from multiple threads.

        // ^ The above is just a massive fuck-you to myself if I misthought this
        // lmao
        if let Some(index) = self.compute_index(cursor) {
            unsafe {
                self.data.add(index).write_volatile(data);
                true
            }
        } else {
            false
        }
    }

    #[inline(always)]
    fn compute_index(&self, cursor: Cursor) -> Option<usize> {
        (cursor.x < self.width && cursor.y < self.height)
            .then_some(cursor.y * self.width + cursor.x)
    }
}
