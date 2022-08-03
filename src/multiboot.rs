/*! License for Multiboot2 Specification Version 2.0
# Multiboot2 Specification

This file documents Multiboot2 Specification, the proposal for the boot sequence standard. This edition documents version 2.0.

Copyright © 1995,96 Bryan Ford <baford@cs.utah.edu>

Copyright © 1995,96 Erich Stefan Boleyn <erich@uruk.org>

Copyright © 1999,2000,2001,2002,2005,2006,2009,2010,2016 Free Software Foundation, Inc.

Permission is granted to make and distribute verbatim copies of this manual provided the copyright notice and this permission notice are preserved on all copies.

Permission is granted to copy and distribute modified versions of this manual under the conditions for verbatim copying, provided also that the entire resulting derived work is distributed under the terms of a permission notice identical to this one.

Permission is granted to copy and distribute translations of this manual into another language, under the above conditions for modified versions.
*/

#[no_mangle]
#[used(linker)]
#[link_section = ".multiboot"]
/// The field ‘magic’ is the magic number identifying the header, which must be the hexadecimal value 0xE85250D6.
pub static MAGIC: u32 = 0xE85250D6;

#[no_mangle]
#[used(linker)]
#[link_section = ".multiboot"]
/// The field ‘architecture’ specifies the Central Processing Unit Instruction Set Architecture. Since ‘magic’ isn’t a palindrome it already specifies the endianness ISAs differing only in endianness recieve the same ID. ‘0’ means 32-bit (protected) mode of i386. ‘4’ means 32-bit MIPS.
pub static ARCHITECTURE: u32 = 0;

#[no_mangle]
#[used(linker)]
#[link_section = ".multiboot"]
/// The field ‘header_length’ specifies the Length of Multiboot2 header in bytes including magic fields.
pub static HEADER_LENGTH: u32 = 16 + EndTag.size() + InformationRequestTag.size();

#[no_mangle]
#[used(linker)]
#[link_section = ".multiboot"]
/// The field ‘checksum’ is a 32-bit unsigned value which, when added to the other magic fields (i.e. ‘magic’, ‘architecture’ and ‘header_length’), must have a 32-bit unsigned sum of zero.
pub static CHECKSUM: u32 = ARCHITECTURE
    .wrapping_add(HEADER_LENGTH)
    .wrapping_add(MAGIC)
    .wrapping_neg();

#[no_mangle]
#[used(linker)]
#[link_section = ".multiboot"]
pub static InformationRequestTag: InformationRequest<1> = InformationRequest::new([8]);

// #[no_mangle]
// #[used(linker)]
// #[link_section = ".multiboot"]
// pub static FrameBufferTag: FrameBuffer = FrameBuffer::new(0, 0, 0);

/** Add other tags up here */

#[no_mangle]
#[used(linker)]
#[link_section = ".multiboot"]
pub static EndTag: Empty = Empty::new();

#[repr(u16)]
/// Tags constitutes a buffer of structures following each other padded when necessary in order for each tag to start at 8-bytes aligned address. Tags are terminated by a tag of type ‘0’ and size ‘8’. Every structure has following format:
/// ```text
///         +-------------------+
/// u16     | type              |
/// u16     | flags             |
/// u32     | size              |
///         +-------------------+
/// ```
/// ‘type’ is divided into 2 parts. Lower contains an identifier of contents of the rest of the tag. ‘size’ contains the size of tag including header fields. If bit ‘0’ of ‘flags’ (also known as ‘optional’) is set, the bootloader may ignore this tag if it lacks relevant support. Tags are terminated by a tag of type ‘0’ and size ‘8’.
enum TagTypes {
    /// The terminating tag.
    Empty = 0,
    InformationRequest = 1,
    Address = 2,
    EntryAddress = 3,
    Flags = 4,
    FrameBuffer = 5,
    ModuleAlignment = 6,
    EfiBootServices = 7,
    EfiI386EntryAddress = 8,
    EfiAmd64EntryAddress = 9,
    RelocatableHeader = 10,
}

#[repr(u16)]
enum TagFlags {
    Optional = 0,
    /// NonOptional does not strictly say the tag must be provided by the bootloader, but
    /// only acts as a suggestion.
    NonOptional = 1,
}

macro_rules! impl_tags {
        ($(#[$($t:tt)*])* $v:vis struct $a:ident {
            tag_type: TagTypes,
            flags: TagFlags,
            size: u32,
            $($b:ident: $c:ty,)*
        }) => {
            $(#[$($t)*])*
            $v struct $a {
                tag_type: TagTypes,
                flags: TagFlags,
                size: u32,
                $($b: $c),*
            }

            impl $a {
                const fn new($($b: $c),*) -> Self {
                    $a {
                        tag_type: TagTypes::$a,
                        flags: TagFlags::NonOptional,
                        size: core::mem::size_of::<Self>() as u32,
                        $($b),*
                    }
                }

                const fn size(&self) -> u32 {
                    core::mem::size_of::<Self>() as u32
                }
            }
        };

        ($(#[$($t:tt)*])* $v:vis struct $a:ident <const $g:ident: $gt:ty> {
            tag_type: TagTypes,
            flags: TagFlags,
            size: u32,
            $($b:ident: $c:ty,)*
        }) => {
            $(#[$($t)*])*
            $v struct $a <const $g: $gt> {
                tag_type: TagTypes,
                flags: TagFlags,
                size: u32,
                $($b: $c),*
            }

            impl<const $g: $gt> $a<$g> {
                const fn new($($b: $c),*) -> Self {
                    $a {
                        tag_type: TagTypes::$a,
                        flags: TagFlags::NonOptional,
                        size: core::mem::size_of::<Self>() as u32,
                        $($b),*
                    }
                }

                const fn size(&self) -> u32 {
                    core::mem::size_of::<Self>() as u32
                }

            }
        };
    }

impl_tags! {
    #[repr(C)]
    #[repr(align(8))]
    pub struct Empty {
        tag_type: TagTypes,
        flags: TagFlags,
        size: u32,
    }
}

impl_tags! {
    #[repr(C)]
    #[repr(align(8))]
    /// ```text
    ///         +-------------------+
    /// u16     | type = 1          |
    /// u16     | flags             |
    /// u32     | size              |
    /// u32[n]  | mbi_tag_types     |
    ///         +-------------------+
    /// ```
    ///
    /// ‘mbi_tag_types’ is an array of u32’s, each one representing an information request.
    ///
    /// If this tag is present and ‘optional’ is set to ‘0’, the bootloader must support the requested tag and be able to provide relevant information to the image if it is available. If the bootloader does not understand the meaning of the requested tag it must fail with an error. However, if it supports a given tag but the information conveyed by it is not available the bootloader does not provide the requested tag in the Multiboot2 information structure and passes control to the loaded image as usual.
    ///
    /// Note: The above means that there is no guarantee that any tags of type ‘mbi_tag_types’ will actually be present. E.g. on a videoless system even if you requested tag ‘8’ and the bootloader supports it, no tags of type ‘8’ will be present in the Multiboot2 information structure.
    pub struct InformationRequest<const N: usize> {
        tag_type: TagTypes,
        flags: TagFlags,
        size: u32,
        mbi_tag_types: [u32; N],
    }
}

impl_tags! {
    #[repr(C)]
    #[repr(align(8))]
    /// ```text
    ///         +-------------------+
    /// u16     | type = 2          |
    /// u16     | flags             |
    /// u32     | size              |
    /// u32     | header_addr       |
    /// u32     | load_addr         |
    /// u32     | load_end_addr     |
    /// u32     | bss_end_addr      |
    ///         +-------------------+
    /// ```
    ///
    /// All of the address fields in this tag are physical addresses. The meaning of each is as follows:
    ///
    /// header_addr
    ///
    /// Contains the address corresponding to the beginning of the Multiboot2 header — the physical memory location at which the magic value is supposed to be loaded. This field serves to synchronize the mapping between OS image offsets and physical memory addresses.
    ///
    /// load_addr
    ///
    /// Contains the physical address of the beginning of the text segment. The offset in the OS image file at which to start loading is defined by the offset at which the header was found, minus (header_addr - load_addr). load_addr must be less than or equal to header_addr.
    ///
    /// Special value -1 means that the file must be loaded from its beginning.
    ///
    /// load_end_addr
    ///
    /// Contains the physical address of the end of the data segment. (load_end_addr - load_addr) specifies how much data to load. This implies that the text and data segments must be consecutive in the OS image; this is true for existing a.out executable formats. If this field is zero, the boot loader assumes that the text and data segments occupy the whole OS image file.
    ///
    /// bss_end_addr
    ///
    /// Contains the physical address of the end of the bss segment. The boot loader initializes this area to zero, and reserves the memory it occupies to avoid placing boot modules and other data relevant to the operating system in that area. If this field is zero, the boot loader assumes that no bss segment is present.
    ///
    /// Note: This information does not need to be provided if the kernel image is in ELF format, but it must be provided if the image is in a.out format or in some other format. When the address tag is present it must be used in order to load the image, regardless of whether an ELF header is also present. Compliant boot loaders must be able to load images that are either in ELF format or contain the address tag embedded in the Multiboot2 header.
    pub struct Address {
        tag_type: TagTypes,
        flags: TagFlags,
        size: u32,
        header_addr: u32,
        load_addr: u32,
        load_end_addr: u32,
        bss_end_addr: u32,
    }
}

impl_tags! {
    #[repr(C)]
    #[repr(align(8))]
    /// ```text
    ///         +-------------------+
    /// u16     | type = 3          |
    /// u16     | flags             |
    /// u32     | size              |
    /// u32     | entry_addr        |
    ///         +-------------------+
    /// ```
    ///
    /// All of the address fields in this tag are physical addresses. The meaning of each is as follows:
    ///
    /// entry_addr
    ///
    /// The physical address to which the boot loader should jump in order to start running the operating system.
    pub struct EntryAddress {
        tag_type: TagTypes,
        flags: TagFlags,
        size: u32,
        header_addr: u32,
        load_addr: u32,
        load_end_addr: u32,
        bss_end_addr: u32,
    }
}

impl_tags! {
    #[repr(C)]
    #[repr(align(8))]
    /// ```text
    ///         +-------------------+
    /// u16     | type = 8          |
    /// u16     | flags             |
    /// u32     | size              |
    /// u32     | entry_addr        |
    ///         +-------------------+
    /// ```
    ///
    /// All of the address fields in this tag are physical addresses. The meaning of each is as follows:
    ///
    /// entry_addr
    ///
    /// The physical address to which the boot loader should jump in order to start running EFI i386 compatible operating system code.
    ///
    /// This tag is taken into account only on EFI i386 platforms when Multiboot2 image header contains EFI boot services tag. Then entry point specified in ELF header and the entry address tag of Multiboot2 header are ignored.
    pub struct EfiI386EntryAddress {
        tag_type: TagTypes,
        flags: TagFlags,
        size: u32,
        entry_addr: u32,
    }
}

impl_tags! {
    #[repr(C)]
    #[repr(align(8))]
    /// ```text
    ///         +-------------------+
    /// u16     | type = 9          |
    /// u16     | flags             |
    /// u32     | size              |
    /// u32     | entry_addr        |
    ///         +-------------------+
    /// ```
    ///
    /// All of the address fields in this tag are physical addresses (paging mode is enabled and any memory space defined by the UEFI memory map is identity mapped, hence, virtual address equals physical address; Unified Extensible Firmware Interface Specification, Version 2.6, section 2.3.4, x64 Platforms, boot services). The meaning of each is as follows:
    ///
    /// entry_addr
    ///
    /// The physical address to which the boot loader should jump in order to start running EFI amd64 compatible operating system code.
    ///
    /// This tag is taken into account only on EFI amd64 platforms when Multiboot2 image header contains EFI boot services tag. Then entry point specified in ELF header and the entry address tag of Multiboot2 header are ignored.
    pub struct EfiAmd64EntryAddress {
        tag_type: TagTypes,
        flags: TagFlags,
        size: u32,
        entry_addr: u32,
    }
}

impl_tags! {
    #[repr(C)]
    #[repr(align(8))]
    /// ```text
    ///         +-------------------+
    /// u16     | type = 4          |
    /// u16     | flags             |
    /// u32     | size = 12         |
    /// u32     | console_flags     |
    ///         +-------------------+
    /// ```
    ///
    /// If this tag is present and bit 0 of ‘console_flags’ is set at least one of supported consoles must be present and information about it must be available in mbi. If bit ‘1’ of ‘console_flags’ is set it indicates that the OS image has EGA text support.
    pub struct Flags {
        tag_type: TagTypes,
        flags: TagFlags,
        size: u32,
        console_flags: u32,
    }
}

impl_tags! {
    #[repr(C)]
    #[repr(align(8))]
    /// ```text
    ///         +-------------------+
    /// u16     | type = 5          |
    /// u16     | flags             |
    /// u32     | size = 20         |
    /// u32     | width             |
    /// u32     | height            |
    /// u32     | depth             |
    ///         +-------------------+
    /// ```
    ///
    /// This tag specifies the preferred graphics mode. If this tag is present bootloader assumes that the payload has framebuffer support. Note that that is only a recommended mode by the OS image. Boot loader may choose a different mode if it sees fit.
    ///
    /// The meaning of each is as follows:
    ///
    /// width
    ///
    /// Contains the number of the columns. This is specified in pixels in a graphics mode, and in characters in a text mode. The value zero indicates that the OS image has no preference.
    ///
    /// height
    ///
    /// Contains the number of the lines. This is specified in pixels in a graphics mode, and in characters in a text mode. The value zero indicates that the OS image has no preference.
    ///
    /// depth
    ///
    /// Contains the number of bits per pixel in a graphics mode, and zero in a text mode. The value zero indicates that the OS image has no preference.
    pub struct FrameBuffer {
        tag_type: TagTypes,
        flags: TagFlags,
        size: u32,
        height: u32,
        weight: u32,
        depth: u32,
    }
}

impl_tags! {
    #[repr(C)]
    #[repr(align(8))]
    /// ```text
    ///         +-------------------+
    /// u16     | type = 6          |
    /// u16     | flags             |
    /// u32     | size = 8          |
    ///         +-------------------+
    /// ```
    ///
    /// If this tag is present modules must be page aligned.
    pub struct ModuleAlignment {
        tag_type: TagTypes,
        flags: TagFlags,
        size: u32,
    }
}

impl_tags! {
    #[repr(C)]
    #[repr(align(8))]
    /// ```text
    ///         +-------------------+
    /// u16     | type = 7          |
    /// u16     | flags             |
    /// u32     | size = 8          |
    ///         +-------------------+
    /// ```
    ///
    /// This tag indicates that payload supports starting without terminating boot services.
    pub struct EfiBootServices {
        tag_type: TagTypes,
        flags: TagFlags,
        size: u32,
    }
}

impl_tags! {
    #[repr(C)]
    #[repr(align(8))]
    /// ```text
    ///         +-------------------+
    /// u16     | type = 10         |
    /// u16     | flags             |
    /// u32     | size = 24         |
    /// u32     | min_addr          |
    /// u32     | max_addr          |
    /// u32     | align             |
    /// u32     | preference        |
    ///         +-------------------+
    /// ```
    ///
    /// This tag indicates that image is relocatable.
    ///
    /// The meaning of each field is as follows:
    ///
    /// min_addr
    ///
    /// Lowest possible physical address at which image should be loaded. The bootloader cannot load any part of image below this address.
    ///
    /// max_addr
    ///
    /// Highest possible physical address at which loaded image should end. The bootloader cannot load any part of image above this address.
    ///
    /// align
    ///
    /// Image alignment in memory, e.g. 4096.
    ///
    /// preference
    ///
    /// It contains load address placement suggestion for boot loader. Boot loader should follow it. ‘0’ means none, ‘1’ means load image at lowest possible address but not lower than min_addr and ‘2’ means load image at highest possible address but not higher than max_addr.
    pub struct RelocatableHeader {
        tag_type: TagTypes,
        flags: TagFlags,
        size: u32,
        min_addr: u32,
        max_addr: u32,
        align: u32,
        preference: u32,
    }
}
