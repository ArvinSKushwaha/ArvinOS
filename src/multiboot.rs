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

mod multiboot2_tags {
    #[repr(u16)]
    enum TagTypes {
        InformationRequest = 1,
        Address = 2,
    }

    #[repr(u16)]
    enum Flags {
        Optional = 0,
        NonOptional = 1,
    }

    macro_rules! impl_tags {
        ($(#[$($t:tt)*])* $v:vis struct $a:ident {
            tag_type: TagTypes,
            flags: Flags,
            $($b:ident: $c:ty,)*
        }) => {
            $(#[$($t)*])*
            $v struct $a {
                tag_type: TagTypes,
                flags: Flags,
                $($b: $c),*
            }

            impl $a {
                fn new($($b: $c),*) -> Self {
                    $a {
                        tag_type: TagTypes::$a,
                        flags: Flags::NonOptional,
                        $($b),*
                    }
                }
            }
        };

        ($(#[$($t:tt)*])* $v:vis struct $a:ident <$($($g:tt)+$(:$gt:ty)?),*> {
            tag_type: TagTypes,
            flags: Flags,
            $($b:ident: $c:ty,)*
        }) => {
            $(#[$($t)*])*
            $v struct $a <$($($g)+$(:$gt)?),*> {
                tag_type: TagTypes,
                flags: Flags,
                $($b: $c),*
            }

            impl<$($($g)+$(:$gt)*)*> $a<$($($g)+)*> {
                fn new($($b: $c),*) -> Self {
                    $a {
                        tag_type: TagTypes::$a,
                        flags: Flags::NonOptional,
                        $($b),*
                    }
                }
            }
        };
    }

    impl_tags! {
        #[repr(C)]
        #[repr(align(8))]
        pub struct InformationRequest<const N: usize> {
            tag_type: TagTypes,
            flags: Flags,
            size: u32,
            mbi_tag_types: [u32; N],
        }
    }

    impl_tags! {
        #[repr(C)]
        #[repr(align(8))]
        pub struct Address {
            tag_type: TagTypes,
            flags: Flags,
            size: u32,
            header_addr: u32,
            load_addr: u32,
            load_end_addr: u32,
            bss_end_addr: u32,
        }
    }
}
