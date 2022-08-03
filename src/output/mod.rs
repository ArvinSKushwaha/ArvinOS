//! This module provides support for outputting data

use self::tty::WRITER;

pub mod tty;

pub fn setup_text_buffer() {
    unsafe { WRITER.initialize(0xb8000, 80, 25).ok() };
}
