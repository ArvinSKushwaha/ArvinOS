/*!
This code has been adapted from "multiboot.h", for which the license is given as follows:

multiboot.h - Multiboot header file.
Copyright (C) 1999,2003,2007,2008,2009,2010  Free Software Foundation, Inc.

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to
deal in the Software without restriction, including without limitation the
rights to use, copy, modify, merge, publish, distribute, sublicense, and/or
sell copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in
all copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT.  IN NO EVENT SHALL ANY
DEVELOPER OR DISTRIBUTOR BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY,
WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR
IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.
*/

//****** Set Header ******//
#[no_mangle]
#[used(linker)]
#[link_section = ".multiboot"]
static HEADER: Header = HeaderBuilder::new(
    HeaderFlags::PAGE_ALIGN
        .union(HeaderFlags::MEMORY_INFO)
        .union(HeaderFlags::VIDEO_MODE),
)
.video_settings(VideoModeSettings {
    mode_type: VideoModeType::Ega,
    width: 80,
    height: 25,
    depth: 0,
})
.build();

//****** Define Types ******//

use bitflags::bitflags;

bitflags! {
    #[repr(transparent)]
    struct HeaderFlags: u32 {
        const PAGE_ALIGN = 0x00000001;
        const MEMORY_INFO = 0x00000002;
        const VIDEO_MODE = 0x00000004;
        const AOUT_KLUDGE = 0x00010000;
    }
}

#[repr(C, align(4))]
struct Header {
    magic: u32,
    flags: HeaderFlags,
    checksum: u32,

    // Only if AOUT_KLUDGE is set
    aout_kludge: AoutKludge,

    // Only if VIDEO_MODE is set
    video_settings: VideoModeSettings,
}

#[repr(C)]
struct AoutKludge {
    header_addr: u32,
    load_addr: u32,
    load_end_addr: u32,
    bss_end_addr: u32,
    entry_addr: u32,
}

#[repr(u32)]
enum VideoModeType {
    Linear = 0,
    Ega = 1,
}

#[repr(C)]
struct VideoModeSettings {
    mode_type: VideoModeType,
    width: u32,
    height: u32,
    depth: u32,
}

struct HeaderBuilder {
    flags: HeaderFlags,
    aout_kludge: Option<AoutKludge>,
    video_settings: Option<VideoModeSettings>,
}

/** The magic field should contain this. */
const HEADER_MAGIC: u32 = 0x1BADB002;

/** This should be in %eax. */
pub const BOOTLOADER_MAGIC: u32 = 0x2BADB002;

/** Alignment of multiboot modules. */
const MOD_ALIGN: u32 = 0x00001000;

/** Alignment of the multiboot info structure. */
const INFO_ALIGN: u32 = 0x00000004;

impl HeaderBuilder {
    const fn new(flags: HeaderFlags) -> Self {
        Self {
            flags,
            aout_kludge: None,
            video_settings: None,
        }
    }

    const fn aout_kludge(mut self, aout_kludge: AoutKludge) -> Self {
        if self.flags.contains(HeaderFlags::AOUT_KLUDGE) {
            self.aout_kludge = Some(aout_kludge);
        }

        self
    }

    const fn video_settings(mut self, video_settings: VideoModeSettings) -> Self {
        if self.flags.contains(HeaderFlags::VIDEO_MODE) {
            self.video_settings = Some(video_settings);
        }

        self
    }

    const fn build(self) -> Header {
        let aout_kludge = if let Some(aout_kludge) = self.aout_kludge {
            aout_kludge
        } else {
            AoutKludge {
                load_addr: 0,
                load_end_addr: 0,
                entry_addr: 0,
                header_addr: 0,
                bss_end_addr: 0,
            }
        };

        let video_settings = if let Some(video_settings) = self.video_settings {
            video_settings
        } else {
            VideoModeSettings {
                mode_type: VideoModeType::Ega,
                width: 0,
                depth: 0,
                height: 0,
            }
        };

        Header {
            magic: HEADER_MAGIC,
            flags: self.flags,
            checksum: self.flags.bits().wrapping_add(HEADER_MAGIC).wrapping_neg(),
            aout_kludge,
            video_settings,
        }
    }
}

/* Flags to be set in the ’flags’ member of the multiboot info structure. */

/** is there basic lower/upper memory information? */
const INFO_MEMORY: u32 = 0x00000001;
/** is there a boot device set? */
const INFO_BOOTDEV: u32 = 0x00000002;
/** is the command-line defined? */
const INFO_CMDLINE: u32 = 0x00000004;
/** are there modules to do something with? */
const INFO_MODS: u32 = 0x00000008;

/* These next two are mutually exclusive */

/** is there a symbol table loaded? */
const INFO_AOUT_SYMS: u32 = 0x00000010;
/** is there an ELF section header table? */
const INFO_ELF_SHDR: u32 = 0x00000020;

/** is there a full memory map? */
const INFO_MEM_MAP: u32 = 0x00000040;

/** Is there drive info? */
const INFO_DRIVE_INFO: u32 = 0x00000080;

/** Is there a config table? */
const INFO_CONFIG_TABLE: u32 = 0x00000100;

/** Is there a boot loader name? */
const INFO_BOOT_LOADER_NAME: u32 = 0x00000200;

/** Is there a APM table? */
const INFO_APM_TABLE: u32 = 0x00000400;

/** Is there video information? */
const INFO_VBE_INFO: u32 = 0x00000800;
const INFO_FRAMEBUFFER_INFO: u32 = 0x00001000;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct AoutSymbolTable {
    pub tabsize: u32,
    pub strsize: u32,
    pub addr: u32,
    pub reserved: u32,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct ElfSectionHeaderTable {
    pub num: u32,
    pub size: u32,
    pub addr: u32,
    pub shndx: u32,
}

#[repr(C)]
pub union SymbolTable {
    pub aout: AoutSymbolTable,
    pub elf: ElfSectionHeaderTable,
}

#[repr(C, u8)]
pub enum FramebufferType {
    Indexed {
        framebuffer_palette_addr: u32,
        framebuffer_palette_num_colors: u16,
    } = 0,
    Rgb {
        framebuffer_red_field_position: u8,
        framebuffer_red_mask_size: u8,
        framebuffer_green_field_position: u8,
        framebuffer_green_mask_size: u8,
        framebuffer_blue_field_position: u8,
        framebuffer_blue_mask_size: u8,
    } = 1,
    EgaText = 2,
}

#[repr(C, align(4))]
pub struct Info {
    /** Multiboot info version number */
    pub flags: u32,

    /* Available memory from BIOS */
    pub mem_lower: u32,
    pub mem_upper: u32,

    /** "root" partition */
    pub boot_device: u32,

    /** Kernel command line */
    pub cmdline: u32,

    /** Boot-Module list */
    pub mods_count: u32,
    pub mods_addr: u32,

    /** Memory Mapping buffer */
    pub mmap_length: u32,
    pub mmap_addr: u32,

    pub u: SymbolTable,

    /** Drive Info buffer */
    pub drives_length: u32,
    pub drives_addr: u32,

    /** ROM configuration table */
    pub config_table: u32,

    /** Boot Loader Name */
    pub boot_loader_name: u32,

    /** APM table */
    pub apm_table: u32,

    /** Video */
    pub vbe_control_info: u32,
    pub vbe_mode_info: u32,
    pub vbe_mode: u16,
    pub vbe_interface_seg: u16,
    pub vbe_interface_off: u16,
    pub vbe_interface_len: u16,

    pub framebuffer_addr: u64,
    pub framebuffer_pitch: u32,
    pub framebuffer_width: u32,
    pub framebuffer_height: u32,
    pub framebuffer_bpp: u8,

    pub framebuffer_info: FramebufferType,
}

#[repr(C)]
struct Color {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}

#[repr(u32)]
pub enum MmapEntryType {
    Available = 1,
    Reserved = 2,
    AcpiReclaimable = 3,
    Nvs = 4,
    BadRam = 5,
}

#[repr(C, packed)]
pub struct MmapEntry {
    pub size: u32,
    pub addr: u64,
    pub len: u64,
    pub mmap_type: MmapEntryType,
}

#[repr(C)]
pub struct ModList {
    /** the memory used goes from bytes ’mod_start’ to ’mod_end-1’ inclusive */
    pub mod_start: u32,
    pub mod_end: u32,

    /** Module command line */
    pub cmdline: u32,

    /** padding to take it to 16 bytes (must be zero) */
    pub pad: u32,
}

#[repr(C)]
pub struct ApmInfo {
    pub version: u16,
    pub cseg: u16,
    pub offset: u32,
    pub cseg_16: u16,
    pub dseg: u16,
    pub flags: u16,
    pub cseg_len: u16,
    pub cseg_16_len: u16,
    pub dseg_len: u16,
}
