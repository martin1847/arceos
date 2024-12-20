#![cfg_attr(feature = "axstd", no_std)]
#![cfg_attr(feature = "axstd", no_main)]
#![feature(asm_const)]

extern crate alloc;

use core::mem::size_of;

use axhal::mem::{phys_to_virt, virt_to_phys};
// #[cfg(feature = "axstd")]
#[macro_use]
#[cfg(feature = "axstd")]
extern crate axstd as std;

use axmm::{kernel_aspace, new_kernel_aspace};
use axstd::println;
use axstd::{os::arceos::modules::axlog::warn, print, thread};
use loader::init_app_stack;
// use loader::{create_file, load_user_app};
// use std::os::arceos::modules::axhal::mem::phys_to_virt;

// use std::fs::File;
// use std::io::{self, prelude::*};

mod loader;
// #[cfg(feature = "use-ramfs")]
// mod ramfs;

const PFLASH_START: usize = 0xffff_ffc0_2200_0000;

/// Physical address for pflash#1
const PA_PFLASH_START: usize = 0x2200_0000;

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

const KERNEL_STACK_SIZE: usize = 0x40000; // 256 KiB

#[cfg_attr(feature = "axstd", no_mangle)]
fn main() {
    println!("Multi-task is starting Test Paging...");

    register_abi(SYS_HELLO, abi_hello as usize);
    register_abi(SYS_PUTCHAR, abi_putchar as usize);
    register_abi(SYS_TERMINATE, shutdown as usize);

    // A new address space for app.
    // "/sbin/hello_c.bin"
    // let mut kspace = kernel_aspace().lock(); // new_kernel_aspace().unwrap();

    // 再映射一次，找不到
    let mut kspace = new_kernel_aspace().unwrap(); // new_kernel_aspace().unwrap();
    let entry = match loader::load_user_app("/sbin/hello", &mut kspace) {
        Ok(e) => e,
        Err(err) => panic!("Cannot load app! {:?}", err),
    };
    warn!("entry {:#x} ", entry);
    // let apps_size = 32; // Dangerous!!! We need to get accurate size of apps.

    // Init user stack.
    let ustack_pointer = init_app_stack(&mut kspace).unwrap();
    warn!(
        "New user address space: {:#x?} , app_stack -> {:?}",
        kspace, ustack_pointer
    );

    // let task: thread::JoinHandle<()> = thread::spawn(move || unsafe {
    //     println!("New Task Exec payload  task  !");
    //     core::arch::asm!(
    //         "
    //         mv sp , {ustack_pointer}
    //         mv      t2, {entry}
    //         jalr   t2",
    //         entry = in(reg)  entry,
    //         ustack_pointer = in(reg) ustack_pointer.as_usize(),
    //         // options(noreturn),
    //     );
    // });
    // task.join();
    // // let entry_fn = unsafe { &*(entry as *const _ as *const dyn FnOnce) };
    let mut task = axtask::TaskInner::new(
        move || unsafe {
            // sli      a7, {run_start}
            core::arch::asm!(
                "
                mv sp , {ustack_pointer}
                mv      t2, {entry}
                jalr   t2",
                entry = in(reg)  entry,
                ustack_pointer = in(reg) ustack_pointer.as_usize(),
                // options(noreturn),
            )
        },
        "appboot".into(),
        KERNEL_STACK_SIZE,
    );
    // let task_stack_top = task.kernel_stack_top().unwrap();
    // init_app_stack(task_stack_top);
    let task = axtask::spawn_task(task);
    // task.join();
    let exit_code = task.join();
    warn!("kernel exit [{:?}] normally!", exit_code);
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
    print!("{c}");
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

fn run_code_at_start(code: &[u8], app_size: usize) {
    // shutdown();
    println!("Try Execute app   ...");
    let run_code = unsafe { core::slice::from_raw_parts_mut(RUN_START as *mut u8, app_size) };
    // run_code.copy_from_slice(&code[0..8]);
    run_code.copy_from_slice(code);
    println!("run code {:?}; address [{:?}]", run_code, run_code.as_ptr());
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

fn load_from_pflash() {
    // assert_eq!(addr, 0x2200_0000.into(), "Now we ONLY handle pflash#2.");
    let mapping_flags = axhal::paging::MappingFlags::from_bits(0xf).unwrap();
    // Passthrough-Mode
    let va_pf = phys_to_virt(PA_PFLASH_START.into());

    // let sz = size_of::<PartHeader>();
    // let mut buf = [0 as u8; 8];
    // kernel_aspace()
    //     .lock()
    //     .read(PA_PFLASH_START.into(), &mut buf);
    // println!("apps_start: {:?} , buf {:?}", va_pf, buf);
    //     .lock()
    //     .map_linear(va_pf, PA_PFLASH_START.into(), 4096 * 100, mapping_flags);

    // let mut apps_start = PFLASH_START; //without paging
    let mut apps_start = va_pf.as_usize();
    println!("apps_start: {:x}", apps_start);
    // unsafe {
    //     let ptr = apps_start as *const u32;
    //     println!(
    //         "Try to access dev region [{:#X}], got {:#X}",
    //         apps_start, *ptr
    //     );
    //     let magic = core::mem::transmute::<u32, [u8; 4]>(*ptr);
    //     println!(
    //         "Got pflash magic: {}",
    //         core::str::from_utf8(&magic).unwrap()
    //     );
    // }

    let mut app_num = 0;
    loop {
        let part = unsafe { &*(apps_start as *const PartHeader) };
        // let part_ptr = apps_start as *const u32;
        // let part = unsafe {
        //     core::mem::transmute::<u32, PartHeader>(*part_ptr);
        // };
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

        println!("Load payload {} ok !!\n", app_num);

        // A new address space for app.
        // let mut uspace = axmm::new_kernel_aspace().unwrap();
        // let file = "tmp_code.bin";
        // create_file(file, code).unwrap();
        // let entry = loader::load_user_app(code, &mut uspace).unwrap();

        // warn!("entry {:#x} ", entry);
        // // let entry_fn = unsafe { &*(entry as *const _ as *const dyn FnOnce) };
        // let mut task = axtask::TaskInner::new(
        //     move || unsafe {
        //         // sli      a7, {run_start}
        //         core::arch::asm!(
        //             "mv      t2, {entry}
        //             jalr   t2",
        //             entry = in(reg)  entry,
        //             // options(noreturn),
        //         )
        //     },
        //     "appboot".into(),
        //     KERNEL_STACK_SIZE,
        // );
        // let task = axtask::spawn_task(task);
        // task.join();

        // let task: thread::JoinHandle<()> = thread::spawn(move || {
        //     println!("New Task Exec payload  task {} ok !!\n", app_num);
        //     run_code_at_start(code, app_size);
        //     // let elf = load_elf_phdrs(code).unwrap();
        // });
        // task.join();
        println!("Exec payload  task {} ok !!\n", app_num);
    }
}
