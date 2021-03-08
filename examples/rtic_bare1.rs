//! bare1.rs
//!
//! Inspecting the generated assembly
//!
//! What it covers
//! - Rust panic on arithmetics
//! - assembly calls and inline assembly

#![no_main]
#![no_std]

use panic_semihosting as _;
use stm32f4;

#[rtic::app(device = stm32f4)]
const APP: () = {
    #[init]
    #[inline(never)] // avoid inlining of this function/task
    #[no_mangle] // to strip hash from symbols (easier to read)
    fn init(_cx: init::Context) {
        let mut x = core::u32::MAX - 1;
        loop {
            cortex_m::asm::bkpt();
            x += x.wrapping_add(1);
            cortex_m::asm::bkpt();

            // prevent optimization by read-volatile (unsafe)
            /*unsafe {
                core::ptr::read_volatile(&x);
            }*/
        }
    }
};

// 0. Setup
//    Make sure that your repository is updated (pull from upstream).
//
// 1. Build in debug mode and run the application in vscode (Cortex Debug)
//
//    Continue until you hit a breakpoint.
//
//    Now select OUTPUT and Adapter Output.
//
//    You should have encountered a Rust panic.
//
//    Paste the error message:
//
//    ** My answer here **
//    panicked at 'attempt to add with overflow', examples/rtic_bare1.rs:24:13
//
//    Explain in your own words why the code panic:ed.
//
//    ** My answer here **
//    core::u32::MAX - 1 sets the value of x to the maximum value a 32 unsigned bit can hold - 1 and
//    then starts a loop with an addition of 1 to x, that's a recipe for overflow.
//
//    Commit your answer (bare1_1)
//
// 2. Inspecting what caused the panic.
//
//    Under CALL STACK you find the calls done to reach the panic:
//
//    You can get the same information directly from GDB
//
//    Select the DEBUG CONSOLE and enter the command
//
//    > backtrace
//
//    Paste the backtrace:
//
//    ** My answer here **
/*
    #0  lib::__bkpt () at asm/lib.rs:49
    #1  0x0800104e in cortex_m::asm::bkpt () at /home/decending/.cargo/registry/src/github.com-1ecc6299db9ec823/cortex-m-0.7.1/src/asm.rs:15
    #2  rust_begin_unwind (info=0x2000fed8) at /home/decending/.cargo/registry/src/github.com-1ecc6299db9ec823/panic-semihosting-0.5.6/src/lib.rs:92
    #3  0x0800039a in core::panicking::panic_fmt () at /rustc/cb75ad5db02783e8b0222fee363c5f63f7e2cf5b//library/core/src/panicking.rs:92
    #4  0x08000374 in core::panicking::panic () at /rustc/cb75ad5db02783e8b0222fee363c5f63f7e2cf5b//library/core/src/panicking.rs:50
    #5  0x08000ebe in rtic_bare1::init (_cx=...) at /home/decending/Desktop/e7020e_2021/examples/rtic_bare1.rs:24
    #6  0x08000f08 in rtic_bare1::APP::main () at /home/decending/Desktop/e7020e_2021/examples/rtic_bare1.rs:15

*/
//
//    Explain in your own words the chain of calls.
//
//    ** My answer here **
//    This will run from the "bottom up":
//    0# We enter a function with a breakpoint
//    1# Calling function for breakpoint
//    2# We unwind to catch the panic
//    3# We continue to handle the panic
//    4# We start to panic
//    5# We start the init task.
//    6# We call the APP::main task, 
//
//    Commit your answer (bare1_2)
//
// 3. Now let's try to break it down to see what caused the panic.
//
//    Put a breakpoint at line 24 (x += 1;)
//    (Click to the left of the line marker, you get a red dot.)
//
//    Restart the debug session, and continue until you hit the breakpoint.
//
//    What is the value of `x`?
//
//    ** My answer here **
//    x = 4294967294;
//
//    Explain in your own words where this value comes from.
//
//    ** My answer here **
//    This is the value of a 32-bit unsigned integer with the form:
//    11111111111111111111111111111110, which is core::u32::MAX - 1.
//
//    Now continue the program, since you are in a loop
//    the program will halt again at line 24.
//
//    What is the value of `x`?
//
//    Explain in your own words why `x` now has this value.
//
//    ** My answer here **
//    x = 4294967295;
//    4294967294 + 1 = 4294967295
//    This is what the loop is meant to do, so nothing strange here.
//
//    Now continue again.
//
//    At this point your code should panic.
//
//    You can navigate the CALL STACK.
//    Click on rtic_bare::init@0x08.. (24)
//
//    The line leading up to the panic should now be highlighted.
//    So you can locate the precise line which caused the error.
//
//    Explain in your own words why a panic makes sense at this point.
//
//    ** My answer here **
//    Since we're not using any sort of wrapping the new number, which is 4294967296 can't be contained
//    in a u32 datastructure, so this becomes an undefined behaviour and hence the code panics.
//
//    Commit your answer (bare1_3)
//
// 4. Now lets have a look at the generated assembly.
//
//    First restart the debug session and continue to the first halt (line 24).
//
//    Select DEBUG CONSOLE and give the command
//
//    > disassemble
//
//    The current PC (program counter is marked with an arrow)
//    => 0x08000f18 <+20>:	ldr	r0, [sp, #0]
//
//    Explain in your own words what this assembly line does.
//
//    ** My answer here **
//    It'll load the registry sp with an immediate offset of 0 into r0. This should be our x.
//
//    In Cortex Registers (left) you can see the content of `r0`
//
//    What value do you observe?
//
//    ** My answer here **
//    -2 = 0xfffffffe.
//
//    You can also get the register info from GDB directly.
//
//    > register info
//
//    Many GDB commands have short names try `i r`.
//
//    So now, time to move on, one assembly instruction at a time.
//
//    > stepi
//    > disassemble
//
//    Now you should get
//    => 0x08000f1a <+22>:	adds	r0, #1
//
//    Explain in your own words what is happening here.
//
//    ** My answer here **
//    We will add r0 and the immidate value 1 together (x + 1)
//
//    We move to the next assembly instruction:
//
//    > si
//    > i r
//
//    What is the reported value for `r0`
//
//    ** My answer here **
//    0xffffffff = -1
//
//    So far so good.
//
//    We can now continue to the next breakpoint.
//
//    > continue
//    (or in short >c, or press the play button, or press F5, many options here ...)
//    > disassemble
//    (or in short >disass)
//
//    You should now be back at the top of the loop:
//
//    => 0x08000f18 <+20>:	ldr	r0, [sp, #0]
//
//    and the value of `r0` should be -1 (or 0xffffffff in hexadecimal)
//
//    Now we can step an instruction again.
//
//    > si
//    => 0x08000f1a <+22>:	adds	r0, #1
//
//    So far so good, and another one.
//
//    > si
//    => 0x08000f1c <+24>:	bcs.n	0x8000f28 <rtic_bare::init+36>
//
//    lookup the arm instruction set: https://developer.arm.com/documentation/ddi0210/c/Introduction/Instruction-set-summary/Thumb-instruction-summary
//
//    What does BCS do?
//
//    ** My answer here **
//    It'll check for the carry flag and if it is set the instruction will jump to the provided address.
//
//    Now let's see what happens.
//
//    > si
//    => 0x08000f28 <+36>:	movw	r0, #6128	; 0x17f0
//       0x08000f2c <+40>:	movw	r2, #6112	; 0x17e0
//       0x08000f30 <+44>:	movt	r0, #2048	; 0x800
//       0x08000f34 <+48>:	movt	r2, #2048	; 0x800
//       0x08000f38 <+52>:	movs	r1, #28
//       0x08000f3a <+54>:	bl	0x8000346 <_ZN4core9panicking5panic17h6c8437680724f6d0E>
//
//    Explain in your own words where we are heading.
//
//    ** My answer here **
//    We are heading into the panicking system.
//
//    To validate that your answer, let's let the program continue
//
//    > c
//
//    Look in the OUTPUT/Adapter Output console again.
//
//    Explain in your own words what the code
//    0x08000f28 ..  0x08000f38 achieves
//
//    Hint 1, look at the error message?
//    Hint 2, look at the call stack.
//    Hint 3, the code is generated by the Rust compiler to produce the error message.
//            there is no "magic" here, just a compiler generating code...
//
//    ** My answer here **
//    We save the relevant information regarding the panic.
//
//    Commit your answer (bare1_3)
//
// 5. Now we can remove the break point (click the `Remove All Breakpoints`),
//    and instead uncomment the two breakpoint instructions (on lines 23 and 25).
//
//    Close the debug session and press F5 again to re-compile and launch the app.
//
//    Continue until you hit the firs breakpoint.
//
//    The disassembly should look like this:
//
//Release
//       0x08000f18 <+20>:	bl	0x800103e <lib::__bkpt>
//    => 0x08000f1c <+24>:	ldr	r0, [sp, #0]
//       0x08000f1e <+26>:	adds	r0, #1
//       0x08000f20 <+28>:	bcs.n	0x8000f30 <rtic_bare::init+44>
//       0x08000f22 <+30>:	str	r0, [sp, #0]
//       0x08000f24 <+32>:	bl	0x800103e <lib::__bkpt>
//       0x08000f28 <+36>:	mov	r0, r4
//       0x08000f2a <+38>:	bl	0x8000fde <_ZN4core3ptr13read_volatile17hea5ef1c780562e1fE>
//
//    In stable Rust we cannot currently write inline assembly, thus we do a "workaround"
//    and call a function that that contains the assembly instruction.
//
//    In this code:
//       0x08000f18 <+20>:	bl	0x800103e <lib::__bkpt>
//    and
//       0x08000f24 <+32>:	bl	0x800103e <lib::__bkpt>
//
//    In cases, this is not good enough (if we want exact cycle by cycle control).
//    We can overcome this by letting the linker inline the code.
//
//    Let's try this, build and run the code in release mode (Cortex Release).
//    Continue until you hit the first assembly breakpoint.
//
//    The disassembly now should look like this:
//
//    => 0x0800024c <+12>:	bkpt	0x0000
//       0x0800024e <+14>:	adds	r0, #1
//       0x08000250 <+16>:	str	r0, [sp, #4]
//       0x08000252 <+18>:	bkpt	0x0000
//       0x08000254 <+20>:	ldr	r0, [sp, #4]
//       0x08000256 <+22>:	b.n	0x800024c <rtic_bare::init+12>
//
//    Now let's compare the two assembly snippets.
//    We now see that the breakpoints have been inlined (offsets +12, +18).
//
//    But something else also happened here!
//
//    Do you see any way this code may end up in a panic?
//
//    ** My answer here **
//    The program still has the possibility or inevitability of an overflow in it's current implementation.
//
//    So clearly, the "semantics" (meaning) of the program has changed.
//    This is on purpose, Rust adopts "unchecked" (wrapping) additions (and subtractions)
//    by default in release mode (to improve performance).
//
//    The downside, is that programs change meaning. If you intend the operation
//    to be wrapping you can explicitly express that in the code.
//
//    Change the x += 1 to x = x.wrapping_add(1).
//
//    And recompile/run/the code in Debug mode
//
//    Paste the generated assembly:
//
//    ** My answer here **
/*
        => 0x0800024e <+14>:	bkpt	0x0000
           0x08000250 <+16>:	adds	r0, #1
           0x08000252 <+18>:	str	r0, [sp, #4]
           0x08000254 <+20>:	bkpt	0x0000
           0x08000256 <+22>:	ldr	r0, [sp, #4]
           0x08000258 <+24>:	b.n	0x800024c <rtic_bare1::init+12>

*/
//
//    Can this code generate a panic?
//
//    ** My answer here **
//    No, there is no reference to the panic handler.
//
//    Is there now any reference to the panic handler?
//    If not, why is that the case?
//
//    ** My answer here **
//    The reference to the panic handler is not present anymore, which indicates that the
//    compiler have found the code to be unable to generate a panic.
//
//    commit your answers (bare1_5)
//
//    Discussion:
//    In release (optimized) mode the addition is unchecked,
//    so there is a semantic difference here in between
//    the dev and release modes. This is motivated by:
//    1) efficiency, unchecked/wrapping is faster
//    2) convenience, it would be inconvenient to explicitly use
//    wrapping arithmetics, and wrapping is what the programmer
//    typically would expect in any case. So the check
//    in dev/debug mode is just there for some extra safety
//    if your intention is NON-wrapping arithmetics.
//
//    The debug build should have additional code that checks if the addition
//    wraps (and in such case call panic). In the case of the optimized
//    build there should be no reference to the panic handler in the generated
//    binary. Recovering from a panic is in general very hard. Typically
//    the best we can do is to stop and report the error (and maybe restart).
//
//    Later we will demonstrate how we can get guarantees of panic free execution.
//    This is very important to improve reliability.
//
// 6. Now comment out the `read_volatile`.
//
//    Rebuild and run the code in Release mode.
//
//    Dump the generated assembly.
//
//    ** My answer here **
/*       0x08000240 <+0>:	push	{r7, lr}
         0x08000242 <+2>:	mov	r7, sp
      => 0x08000244 <+4>:	bkpt	0x0000
         0x08000246 <+6>:	bkpt	0x0000
         0x08000248 <+8>:	b.n	0x8000244 <rtic_bare1::init+4>
*/
//
//    Where is the local variable stored?
//    What happened, and why is Rust + LLVM allowed to optimize out your code?
//
//    ** My answer here **
//    Since the variable X is no longer used, it has been optimized out
//    to optimize the program. In this case we optimized out the only
//    function.
//
//    Commit your answers (bare1_6)
//
//
// 7. *Optional
//    You can pass additional flags to the Rust `rustc` compiler.
//
//    `-Z force-overflow-checks=off`
//
//    Under this flag, code is never generated for overflow checking even in
//    non optimized (debug/dev) builds.
//    You can enable this flag in the `.cargo/config` file.
//
//    What is now the disassembly of the loop (in debug/dev mode):
//
//    ** your answer here **
//
//    commit your answers (bare1_7)
//
//    Now restore the `.cargo/config` to its original state.
//
// 8. *Optional
//    There is another way to conveniently use wrapping arithmetics
//    without passing flags to the compiler.
//
//    https://doc.rust-lang.org/std/num/struct.Wrapping.html
//
//    Rewrite the code using this approach.
//
//    What is now the disassembly of the code in dev mode?
//
//    ** your answer here **
//
//    What is now the disassembly of the code in release mode?
//
//    ** your answer here **
//
//    commit your answers (bare1_8)
//
//    Final discussion:
//
//    Embedded code typically is performance sensitive, hence
//    it is important to understand how code is generated
//    to achieve efficient implementations.
//
//    Moreover, arithmetics are key to processing of data,
//    so its important that we are in control over the
//    computations. E.g. computing checksums, hashes, cryptos etc.
//    all require precise control over wrapping vs. overflow behavior.
//
//    If you write a library depending on wrapping arithmetics
//    do NOT rely on a compiler flag. (The end user might compile
//    it without this flag enabled, and thus get erroneous results.)
//
