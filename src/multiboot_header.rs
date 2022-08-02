#[no_mangle]
#[used(linker)]
#[link_section = ".multiboot"]
pub static MAGIC: u32 = 0xE85250D6;

#[no_mangle]
#[used(linker)]
#[link_section = ".multiboot"]
pub static ARCHITECTURE: u32 = 0;

#[no_mangle]
#[used(linker)]
#[link_section = ".multiboot"]
pub static HEADER_LENGTH: u32 = 16;

#[no_mangle]
#[used(linker)]
#[link_section = ".multiboot"]
pub static CHECKSUM: u32 = ARCHITECTURE
    .wrapping_add(HEADER_LENGTH)
    .wrapping_add(MAGIC)
    .wrapping_neg();
