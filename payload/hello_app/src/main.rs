// #![feature(asm_const)]
#![no_std]
#![no_main]
// #![feature(naked_functions)]

const SYS_HELLO: usize = 1;
const SYS_PUTCHAR: usize = 2;
const SYS_TERMINATE: usize = 3;

static mut ABI_TABLE_ADDR: usize = 0;

#[no_mangle]
unsafe extern "C" fn _start() {
    // let arg0: u8 = b'C';
    // let mut abi_addr: usize = 0;
    core::arch::asm!("mv {0},a7", out(reg) ABI_TABLE_ADDR);

    hello();
    // 直接string会text段搞出问题。
    puts(b"testa");
    // putchar('z');
    // puts("ABI it is here !!!");
    shutdown();
}

fn abi_call(abi_id: usize, arg0: usize) {
    type FnABI = fn(usize, usize, usize, usize);
    let fn_ptr = unsafe {
        core::mem::transmute::<usize, FnABI>(*((ABI_TABLE_ADDR + abi_id * 8) as *const usize))
    };
    fn_ptr(arg0, 1, 2, 3);
}

fn puts(s: &[u8]) {
    // abi_call(SYS_PUTCHAR, c as usize);
    for c in s {
        abi_call(SYS_PUTCHAR, *c as usize);
    }
}

fn putchar(c: char) {
    abi_call(SYS_PUTCHAR, c as usize);
}

// #[naked]
// unsafe extern "C" fn putchar_nake(c: char) {
//     // Manually handle the parameter in the assembly block
//     core::arch::naked_asm!(
//         // Load the parameter from the stack into a0
//         "ld a0, 0(sp)",         // Load the first argument (c) from the stack into a0

//         // Perform the system call
//         "li a7, {abi_num}", // Load the system call number into a7
//         "jalr a7",                // Make the system call

//         // Return from the function
//         // "ret",                  // Return from the function

//         abi_num = const SYS_PUTCHAR,
//         // options(noreturn)
//     );
// }

fn hello() {
    abi_call(SYS_HELLO, 0);
}

fn shutdown() {
    abi_call(SYS_TERMINATE, 0);
}

// fn putchar2(c: char) {
//     unsafe {
//         abi(SYS_PUTCHAR, c as usize);
//     }
// }
// unsafe fn abi(abi_id: usize, arg0: usize) {
//     type FnABI = fn(usize);
//     let fn_ptr = core::mem::transmute::<usize, FnABI>(*((ENTRY + abi_id * 8) as *const usize));
//     fn_ptr(arg0);
// }

// 汇编要自己处理保存一些寄存器。
// addi    sp, sp, -8
//  sd      a7, 0(sp)
// 函数调用
// ld      a7, 0(sp)
// addi    sp, sp, 8
// core::arch::asm!(
//"mv"mv a7,t2", to,{id}",
//	"slli	t0，t日，3"，
//	"add	t1，a7，t0",
//	"Ld	t1，(t1)"，
//
// "jalr	t1"，
// in("t2") ENTRY, id=in(reg) abi_id, in("a0") arg0,
//	clobber_abi("C"),
//)
use core::panic::PanicInfo;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
