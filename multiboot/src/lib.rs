#![no_std]
#![feature(used_with_arg)]

#[repr(C)]
pub struct Multiboot {
    magic: u32,
    flags: Flags,
    checksum: u32,
}

use bitflags::bitflags;

bitflags! {
    #[repr(transparent)]
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    struct Flags: u32 {
        const PageAlignedBootModules = 1 << 0;
        const MemoryInformation = 1 << 1;
        const VideoModeTable = 1 << 2;
        const CheckFurtherFields = 1 << 16;
    }
}

const MAGIC: u32 = 0x1BADB002;
const FLAGS: Flags = Flags::PageAlignedBootModules
    // .union(Flags::VideoModeTable)
    .union(Flags::MemoryInformation);
const CHECKSUM: u32 = MAGIC.wrapping_add(FLAGS.bits()).wrapping_neg();

#[no_mangle]
#[used(linker)]
#[link_section = ".multiboot"]
pub static BOOT_TAG: Multiboot = Multiboot {
    magic: MAGIC,
    flags: FLAGS,
    checksum: CHECKSUM,
};

#[repr(C)]
pub struct MultibootInfo {
    flags: u32,
    mem_lower: u32,
    mem_upper: u32,
    boot_device: u32,
    cmdline: u32,
    mods_count: u32,
    mods_addr: u32,
    syms: [u32; 4],
    mmap_length: u32,
    mmap_addr: u32,
    drives_length: u32,
    drives_addr: u32,
    config_table: u32,
    boot_loader_name: u32,
    apm_table: u32,
    vbe_control_info: u32,
    vbe_mode_info: u32,
    vbe_mode: u32,
    vbe_interface_seg: u32,
    vbe_interface_off: u32,
    vbe_interface_len: u16,
    framebuffer_addr: u64,
    framebuffer_pitch: u32,
    framebuffer_width: u32,
    framebuffer_height: u32,
    framebuffer_bpp: u32,
    framebuffer_type: u8,
    color_info: [u8; 5],
}

impl MultibootInfo {
    pub unsafe fn load(addr: u32) -> Self {
        core::ptr::read(addr as usize as *const MultibootInfo)
    }
}
