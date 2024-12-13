#![cfg_attr(feature = "axstd", no_std)]
#![cfg_attr(feature = "axstd", no_main)]
#![feature(asm_const)]

use core::mem::size_of;

use axstd::os::arceos::modules::axlog::warn;
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

// app running aspace
// SBI(0x80000000) -> App <- Kernel(0x80200000)
// va_pa_offset: 0xffff_ffc0_0000_0000
const RUN_START: usize = 0xffff_ffc0_8010_0000;

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
        let app_size = u32::from_be_bytes(part.length_be) as usize;

        // println!("field2: {:?} -> len  {}", part.length_be, apps_size);

        let code =
            unsafe { core::slice::from_raw_parts((apps_start + HEAD_SIZE) as *const u8, app_size) };
        apps_start += HEAD_SIZE + app_size;
        println!("content: {:?}: ", code);

        let run_code = unsafe { core::slice::from_raw_parts_mut(RUN_START as *mut u8, app_size) };
        // run_code.copy_from_slice(&code[0..8]);
        run_code.copy_from_slice(code);
        println!("run code {:?}; address [{:?}]", run_code, run_code.as_ptr());

        println!("Load payload {} ok!\n", app_num);

        // shutdown();
        println!("Execute app {}  ...", app_num);

        // execute app
        // unsafe {
        //     core::arch::asm!("
        //         li      t2, {run_start}
        //         jalr    ra,t2
        //         ",
        //     // jalr 绝对跳转，并且设置ra返回地址，支持两个调用
        //             // j       .",//死循环在这里
        //                         run_start = const RUN_START,
        //                     )
        // }
        register_abi(SYS_HELLO, abi_hello as usize);
        register_abi(SYS_PUTCHAR, abi_putchar as usize);
        register_abi(SYS_TERMINATE, shutdown as usize);

        unsafe {
            core::arch::asm!("
                la      a7, {abi_table}
                li      t2, {run_start}
                jalr    t2
                j       .",
                run_start = const RUN_START,
                abi_table = sym ABI_TABLE,
            )
        }
        // // println!("Execute app ...");
        // let arg0: u8 = b'A';

        // execute app
        // execute app
        //     unsafe {
        //         core::arch::asm!("
        // la      a7, {abi_table}
        // li      t2, {run_start}
        // jalr    t2
        // j       .",
        //             run_start = const RUN_START,
        //             abi_table = sym ABI_TABLE,
        //         )
        //     }
        // unsafe {
        //     core::arch::asm!("
        // li      t0, {abi_num}
        // slli    t0, t0, 3
        // la      t1, {abi_table}
        // add     t1, t1, t0
        // ld      t1, (t1)
        // jalr    t1
        // li      t2, {run_start}
        // jalr    t2
        // j       .",
        //         run_start = const RUN_START,
        //         abi_table = sym ABI_TABLE,
        //         //abi_num = const SYS_HELLO,
        //         abi_num = const SYS_TERMINATE,
        //         in("a0") arg0,
        //     )
        // }
    }
}

const SYS_HELLO: usize = 1;
const SYS_PUTCHAR: usize = 2;
const SYS_TERMINATE: usize = 3;

static mut ABI_TABLE: [usize; 16] = [0; 16];

fn register_abi(num: usize, handle: usize) {
    unsafe {
        ABI_TABLE[num] = handle;
    }
}
#[no_mangle]
fn abi_hello() {
    println!("[ABI:Hello] Hello, Apps!");
    // let _ = 9 + 9;
}

#[no_mangle]
fn abi_putchar(arg0: usize) {
    let c = arg0 as u8 as char;
    println!("[ABI:Print] {c}");
    // println!("[ABI:Print2] {c}");
    // let _ = 9 + 9;
    // warn!("abi_putchar 123");
}

const SBI_SET_TIMER: usize = 0x54494D45;
const SBI_CONSOLE_PUTCHAR: usize = 1;
const SBI_CONSOLE_GETCHAR: usize = 2;
// const SBI_SHUTDOWN: usize = 8;
// const SBI_SHUTDOWN: usize = 8;
const SBI_SHUTDOWN: usize = 0x53525354;

/// general sbi call
#[inline(always)]
fn sbi_call(which: usize, arg0: usize, arg1: usize, arg2: usize) -> usize {
    let mut ret;
    unsafe {
        core::arch::asm!(
            "ecall",
            inlateout("x10") arg0 => ret,
            in("x11") arg1,
            in("x12") arg2,
            in("x16") 0,
            in("x17") which,
        );
    }
    ret
}

/// use sbi call to set timer
// pub fn set_timer(timer: usize) {
//     sbi_call(SBI_SET_TIMER, timer, 0, 0);
// }

// /// use sbi call to putchar in console (qemu uart handler)
// pub fn console_putchar(c: usize) {
//     sbi_call(SBI_CONSOLE_PUTCHAR, c, 0, 0);
// }

// /// use sbi call to getchar from console (qemu uart handler)
// pub fn console_getchar() -> usize {
//     sbi_call(SBI_CONSOLE_GETCHAR, 0, 0, 0)
// }

/// use sbi call to shutdown the kernel
fn shutdown() -> ! {
    sbi_call(SBI_SHUTDOWN, 0, 0, 0);
    panic!("It should shutdown!");
}
