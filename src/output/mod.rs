//! This module provides support for outputting data

use multiboot2::{FramebufferTag, FramebufferType};

use self::tty::WRITER;

pub mod tty;

pub fn setup_visuals(framebuffer_info: &FramebufferTag) {
    match &framebuffer_info.buffer_type {
        FramebufferType::Text => setup_vga(framebuffer_info),
        FramebufferType::RGB { red, green, blue } => {
            unsafe {
                (framebuffer_info.address as *mut u8).write_bytes(255, 1000);
            }
            todo!("Figure out how to deal with RGB output");
        }
        FramebufferType::Indexed { palette: _ } => {
            todo!("Figure out how to deal with Indexed output");
        }
    }
}

pub fn setup_headless() {
    WRITER.initialize(0xb8000, 80, 25).unwrap();
}

fn setup_vga(framebuffer_info: &FramebufferTag) {
    WRITER
        .initialize(
            framebuffer_info.address as usize,
            framebuffer_info.width as usize,
            framebuffer_info.height as usize,
        )
        .unwrap();
}
