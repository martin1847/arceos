#![cfg_attr(feature = "axstd", no_std)]
#![cfg_attr(feature = "axstd", no_main)]

use core::mem::size_of;

#[cfg(feature = "axstd")]
use axstd::println;

const PLASH_START: usize = 0xffff_ffc0_2200_0000;

////0xCAFEBABE;
const SALUTE_MAGIC: u32 = u32::from_le_bytes((0xCAFEBABE as u32).to_be_bytes());

#[repr(C)]
struct PartHeader {
    magic: u32,
    length_be: [u8; 4],
}

const HEAD_SIZE: usize = size_of::<PartHeader>();

#[cfg_attr(feature = "axstd", no_mangle)]
fn main() {
    let apps_start = PLASH_START as *const u8;
    // let apps_size = 32; // Dangerous!!! We need to get accurate size of apps.

    println!("Load payload ...");

    // let part_ptr: *const PartHeader = ;

    let part: &_ = unsafe { &*(apps_start as *const PartHeader) };
    assert_eq!(part.magic, SALUTE_MAGIC);
    // println!("field1: {:X} , magic {:X}", part.magic, SALUTE_MAGIC); // Output: 78563412 (little-endian)
    let apps_size = u32::from_be_bytes(part.length_be) as usize;

    // println!("field2: {:?} -> len  {}", part.length_be, apps_size);

    let code =
        unsafe { core::slice::from_raw_parts((PLASH_START + HEAD_SIZE) as *const u8, apps_size) };
    println!("content: {:?}: ", code);

    println!("Load payload ok!");
}
