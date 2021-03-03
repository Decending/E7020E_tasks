//! rtic_bare2.rs
//!
//! Measuring execution time
//!
//! What it covers
//! - Generating documentation
//! - Using core peripherals
//! - Measuring time using the DWT

#![no_main]
#![no_std]

use cortex_m::peripheral::DWT;
use cortex_m_semihosting::hprintln;
use panic_semihosting as _;
use stm32f4;

#[rtic::app(device = stm32f4)]
const APP: () = {
    #[init]
    fn init(mut cx: init::Context) {
        cx.core.DWT.enable_cycle_counter();

        // Reading the cycle counter can be done without `owning` access
        // the DWT (since it has no side effect).
        //
        // Look in the docs:
        // pub fn enable_cycle_counter(&mut self)
        // pub fn get_cycle_count() -> u32
        //
        // Notice the difference in the function signature!

        let start = DWT::get_cycle_count();
        wait(1_000_000);
        let end = DWT::get_cycle_count();

        // notice all printing outside of the section to measure!
        hprintln!("Start {:?}", start).ok();
        hprintln!("End {:?}", end).ok();
        hprintln!("Diff {:?}", end.wrapping_sub(start)).ok();

        wait(100);
    }
};

// burns CPU cycles by just looping `i` times
#[inline(never)]
#[no_mangle]
fn wait(i: u32) {
    for _ in 0..i {
        // no operation (ensured not optimized out)
        cortex_m::asm::nop();
    }
}

// 0. Setup
//
//    > cargo doc --open
//
//    `cargo.doc` will document your crate, and open the docs in your browser.
//    If it does not auto-open, then copy paste the path shown in your browser.
//
//    Notice, it will try to document all dependencies, you may have only one
//    panic handler, so temporarily comment out all but one in `Cargo.toml`.
//
//    In the docs, search (`S`) for DWT, and click `cortex_m::peripheral::DWT`.
//    Read the API docs.
//
// 1. Build and run the application in vscode using (Cortex Debug).
//
//    What is the output in the Adapter Output console?
//    (Notice, it will take a while we loop one million times at only 16 MHz.)
//
//    ** My answer here **
//    Start 233354
//    End 167233437
//    Diff 167000083
//
//    Rebuild and run in (Cortex Release).
//
//    ** My answer here **
//    Start 1590210211
//    End 1594210227
//    Diff 4000016
//
//    Compute the ratio between debug/release optimized code
//    (the speedup).
//
//    ** My answer here **
//    167000083 / 4000016 = 41.75
//    Release is 41.75 times faster than debug, or 4175% faster
//
//    commit your answers (bare2_1)
//
// 2. As seen there is a HUGE difference in between Debug and Release builds.
//    In Debug builds, the compiler preserves all abstractions, so there will
//    be a lot of calls and pointer indirections.
//
//    In Release builds, the compiler strives to "smash" all abstractions into straight
//    line code.
//
//    This is what Rust "zero-cost abstractions" means, not zero execution time but rather,
//    "as good as it possibly gets" (you pay no extra cost for using abstractions at run-time).
//
//    In Release builds, the compiler is able to "specialize" the implementation
//    of each function.
//
//    Let us look in detail at the `wait` function:
//    Place a breakpoint at line 54 (wait). Restart the (Cortex Release) session and
//    look at the generated code.
//
//    > disass
//
//    Dump generated assembly for the "wait" function.
//
//    ** My answer here **
/*
   0x080004a0 <+0>:	push	{r7, lr}
   0x080004a2 <+2>:	mov	r7, sp
   0x080004a4 <+4>:	movw	r0, #16960	; 0x4240
   0x080004a8 <+8>:	movt	r0, #15
   0x080004ac <+12>:	nop
   0x080004ae <+14>:	subs	r0, #1
   0x080004b0 <+16>:	bne.n	0x80004ac <rtic_bare2::wait+12>
=> 0x080004b2 <+18>:	pop	{r7, pc}
*/
//
//    Under the ARM calling convention, r0.. is used as arguments.
//    However in this case, we se that r0 is set by the assembly instructions,
//    before the loop is entered.
//
//    Lookup the two instructions `movw` and `movt` to figure out what happens here.
//
//    Answer in your own words, how they assign r0 to 1000000.
//
//    ** My answer here **
//    movw - This instruction will set the bottom half of the register
//    to 16960, so the bottom half will be: 0100001001000000
//    movt - This instruction will set the top half of the register
//    to 15, so the top half will be: 0000000000001111
//    Put together:
//    00000000000011110100001001000000 (binary) = 1000000 (decimal)
//
//    Commit your answers (bare2_2)
//
// 3. Now add a second call to `wait` (line 42).
//
//    Recompile and run until the breakpoint.
//
//    Dump the generated assembly for the "wait" function.
//
//    ** My answer here **
/*
   0x080004a0 <+0>:	push	{r7, lr}
   0x080004a2 <+2>:	mov	r7, sp
   0x080004a4 <+4>:	nop
   0x080004a6 <+6>:	subs	r0, #1
   0x080004a8 <+8>:	bne.n	0x80004a4 <rtic_bare2::wait+4>
=> 0x080004aa <+10>:	pop	{r7, pc}

*/
//
//    Answer in your own words, why you believe the generated code differs?
//
//    ** My answer here **
//    Because the variable is different, if we change the second call to
//    to contain the same variable we get the same generated code.
//
//    Commit your answers (bare2_3)
