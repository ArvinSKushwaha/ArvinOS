#[derive(Debug, Copy, Clone)]
pub enum FrameBufferType {
    Indexed = 0,
    Rgb = 1,
    Ega = 2,
}

#[repr(packed)]
#[derive(Debug, Copy, Clone)]
pub struct Multiboot {
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
    vbe_mode: u16,
    vbe_interface_seg: u16,
    vbe_interface_off: u16,
    vbe_interface_len: u16,
    framebuffer_addr: u64,
    framebuffer_pitch: u32,
    framebuffer_width: u32,
    framebuffer_height: u32,
    framebuffer_bpp: u8,
    framebuffer_type: u8,
}

#[repr(packed)]
#[derive(Debug, Copy, Clone)]
pub struct FrameBufferPaletteSpecs {
    addr: u32,
    num_colors: u16,
}

#[repr(packed)]
#[derive(Debug, Copy, Clone)]
pub struct FrameBufferPositionSpecs {
    red_field_position: u8,
    red_mask_size: u8,
    green_field_position: u8,
    green_mask_size: u8,
    blue_field_position: u8,
    blue_mask_size: u8,
}

pub union FrameBufferSpecs {
    palette: FrameBufferPaletteSpecs,
    position: FrameBufferPositionSpecs,
}
