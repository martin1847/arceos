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
    // let apps_size = 32; // Dangerous!!! We need to get accurate size of apps.

    let mut apps_start = PLASH_START;
    let mut app_num = 0;
    loop {
        let part: &_ = unsafe { &*(apps_start as *const PartHeader) };
        if part.magic != SALUTE_MAGIC {
            break;
        }
        app_num += 1;
        println!("Load payload app {}...", app_num);

        // let part_ptr: *const PartHeader = ;

        // println!("field1: {:X} , magic {:X}", part.magic, SALUTE_MAGIC); // Output: 78563412 (little-endian)
        let apps_size = u32::from_be_bytes(part.length_be) as usize;

        // println!("field2: {:?} -> len  {}", part.length_be, apps_size);

        let code = unsafe {
            core::slice::from_raw_parts((apps_start + HEAD_SIZE) as *const u8, apps_size)
        };
        apps_start += HEAD_SIZE + apps_size;
        println!("content: {:?}: ", code);

        println!("Load payload {} ok!\n", app_num);
    }
}
