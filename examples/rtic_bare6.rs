//! rtic_bare6.rs
//!
//! Clocking
//!
//! What it covers:
//! - using svd2rust generated API
//! - using the stm32f4xx-hal to set clocks
//! - routing the clock to a PIN for monitoring by an oscilloscope

#![no_main]
#![no_std]

use panic_rtt_target as _;
use rtic::cyccnt::{Instant, U32Ext as _};
use rtt_target::{rprintln, rtt_init_print};
use stm32f4xx_hal::{
    prelude::*,
    stm32::{self, GPIOC, RCC},
};

const OFFSET: u32 = 8_000_000;

#[rtic::app(device = stm32f4xx_hal::stm32, monotonic = rtic::cyccnt::CYCCNT, peripherals = true)]
const APP: () = {
    struct Resources {
        // late resources
        GPIOA: stm32::GPIOA,
    }
    #[init(schedule = [toggle])]
    fn init(cx: init::Context) -> init::LateResources {
        rtt_init_print!();
        rprintln!("init");

        let mut core = cx.core;
        let device = cx.device;

        // Initialize (enable) the monotonic timer (CYCCNT)
        core.DCB.enable_trace();
        core.DWT.enable_cycle_counter();

        // semantically, the monotonic timer is frozen at time "zero" during `init`
        // NOTE do *not* call `Instant::now` in this context; it will return a nonsense value
        let now = cx.start; // the start time of the system

        // Schedule `toggle` to run 8e6 cycles (clock cycles) in the future
        cx.schedule.toggle(now + OFFSET.cycles()).unwrap();

        // setup LED
        // power on GPIOA, RM0368 6.3.11
        device.RCC.ahb1enr.modify(|_, w| w.gpioaen().set_bit());
        // configure PA5 as output, RM0368 8.4.1
        device.GPIOA.moder.modify(|_, w| w.moder5().bits(1));

        clock_out(&device.RCC, &device.GPIOC);

        let rcc = device.RCC.constrain();

        //let _clocks = rcc.cfgr.freeze();

        // Set up the system clock. 48 MHz?
         /*let _clocks = rcc
             .cfgr
             .sysclk(48.mhz())
             .pclk1(24.mhz())
             .freeze(); */

        /*let _clocks = rcc
             .cfgr
             .sysclk(64.mhz())
             .pclk1(64.mhz())
             .pclk2(64.mhz())
             .freeze();*/
        
         let _clocks = rcc
             .cfgr
             .sysclk(84.mhz())
             .pclk1(42.mhz())
             .pclk2(64.mhz())
             .freeze();

        // pass on late resources
        init::LateResources {
            GPIOA: device.GPIOA,
        }
    }

    #[idle]
    fn idle(_cx: idle::Context) -> ! {
        rprintln!("idle");
        loop {
            continue;
        }
    }

    #[task(resources = [GPIOA], schedule = [toggle])]
    fn toggle(cx: toggle::Context) {
        static mut TOGGLE: bool = false;
        rprintln!("toggle  @ {:?}", Instant::now());

        if *TOGGLE {
            cx.resources.GPIOA.bsrr.write(|w| w.bs5().set_bit());
        } else {
            cx.resources.GPIOA.bsrr.write(|w| w.br5().set_bit());
        }

        *TOGGLE = !*TOGGLE;
        cx.schedule.toggle(cx.scheduled + OFFSET.cycles()).unwrap();
    }

    extern "C" {
        fn EXTI0();
    }
};

// see the Reference Manual RM0368 (www.st.com/resource/en/reference_manual/dm00096844.pdf)
// rcc,     chapter 6
// gpio,    chapter 8

fn clock_out(rcc: &RCC, gpioc: &GPIOC) {
    // output MCO2 to pin PC9

    // mco2 	: SYSCLK = 0b00
    // mcopre 	: divide by 4 = 0b110
    //rcc.cfgr
    //    .modify(|_, w| unsafe { w.mco2().bits(0b00).mco2pre().bits(0b110) });
    rcc.cfgr
        .modify(|_, w| unsafe { w.mco2().sysclk().mco2pre().div4() });

    // power on GPIOC, RM0368 6.3.11
    rcc.ahb1enr.modify(|_, w| w.gpiocen().enabled());

    // MCO_2 alternate function AF0, STM32F401xD STM32F401xE data sheet
    // table 9
    // AF0, gpioc reset value = AF0

    // configure PC9 as alternate function 0b10, RM0368 6.2.10
    //gpioc.moder.modify(|_, w| w.moder9().bits(0b10));
    gpioc.moder.modify(|_, w| w.moder9().alternate());

    // otyper reset state push/pull, in reset state (don't need to change)

    // ospeedr 0b11 = very high speed
    gpioc.ospeedr.modify(|_, w| w.ospeedr9().very_high_speed());
}

// 0. Background reading:
//
//    Clock trees:
//    STM32F401xD STM32F401xE, section 3.11
//    We have two AMBA High-performance Buses (APBs)
//    APB1 low speed bus (max freq 42 MHz)
//    APB2 high speed bus (max freq 84 MHz)
//
//    RM0368 Section 6.2
//    Some important/useful clock acronyms and their use:
//
//    SYSCLK - the clock that drives the `core`
//    HCLK   - the clock that drives the AMBA bus(es), memory, DMA, trace unit, etc.
//
//    Typically we set HCLK = SYSCLK / 1 (no pre-scale) for our applications
//
//    FCLK   - Free running clock running at HCLK
//
//    CST    - CoreSystemTimer drives the SysTick counter, HCLK/(1 or 8)
//    PCLK1  - The clock driving the APB1 (<= 42 MHz)
//             Timers on the APB1 bus will be triggered at PCLK1 * 2
//    PCLK2  - The clock driving the APB2 (<= 84 MHz)
//             Timers on the APB2 bus will be triggered at PCLK2
//
//    Configuration:
//
//    The `Cargo.toml` file defines your dependencies.
//
//    [dependencies.stm32f4]
//    version = "0.13.0"
//    features = ["stm32f411", "rt"]
//
//    [dependencies.stm32f4xx-hal]
//    version = "0.8.3"
//    features = ["rt", "stm32f411", "usb_fs"]
//
//    The stm32f411 and f401 is essentially the same chip, the f411 is guaranteed
//    up to 100MHz, but we can "overclock" the f401 to 100MHz if needed.
//
//    The `features = ["stm32f411", "rt"]` selects the target MCU, and
//    "rt" enables functionality for exception handling etc.
//
//    The HAL provides a generic abstraction over the whole stm32f4 family.
//
//    In our configuration we enable "stm32f411" with the "rt" feature
//    and the "usb_fs" (for USB OnTheGo support).
//
//    The HAL re-exports the selected "stm32f411" under the `stm32` path.
//
//    Initialization:
//
//    In the code, we first setup the DWT/CYCCNT for the Monotonic timer,
//    and schedule a task to be run after `OFFSET` number of clock cycles.
//
//    The `device.RCC.constrain()`, gives a default setting for the MCU RCC
//    (Reset and Clock Control) peripheral.
//    `rcc.cfgr.x.freeze()`, freezes the current (default) config.
//
//    What is wrong with the following configurations?
//
//    `rcc.cfgr.sysclk(64.mhz()).pclk1(64.mhz()).pclk2(64.mhz()).freeze()`;
//
//    ** My answer here **
//    pclk1 can only handle up to 42mhz
//
//    `rcc.cfgr.sysclk(84.mhz()).pclk1(42.mhz()).pclk2(64.mhz()).freeze();`
//
//    ** My answer here **
//    the pclk1 and pclk2 configurations (42 and 64) doesn't seem to be compatible with the sysclk.
//    The pclk1 and pclk2 clock speeds need to be sysclk / (1 or 2 or 4 or 8 or 16) to be compatible.
//
//    Start `stm32cubemx` and select or create a project targeting stm32f401.
//    Go to the graphical clock configuration view.
//
//    Try to setup the clock according to:
//
//    `rcc.cfgr.sysclk(64.mhz()).pclk1(64.mhz()).pclk2(64.mhz()).freeze()`;
//
//    What happens?
//
//    ** My answer here **
//    Pclk1 is too fast, it has a limit of 42 mhz
//
//    Try to setup the clock according to:
//
//    What happens?
//
//    `rcc.cfgr.sysclk(84.mhz()).pclk1(42.mhz()).pclk2(64.mhz()).freeze();`
//
//    ** My answer here **
//    Since the pclk1 and pclk2 has to be in relation to sysclk according to:
//    pclk(x) = sysclk / (1 or 2 or 4 or 8 or 16), this makes pclk2 and sysclk incompatible in this case.
//
//    Commit your answers (bare6_0)
//
// 1. In this example you will use RTT.
//
//    > cargo run --example rtic_bare6
//
//    Confirm that your RTT traces the init, idle and led on/off.
//
//    What is the (default) MCU (SYSCLK) frequency?
//
//    ** My answer here **
//    16 mhz
//
//    What is the (default) DWT CYCCNT frequency?
//
//    ** your answer here **
//
//    What is the frequency of blinking?
//
//    ** My answer here **
//    8 000 000 / (16 * 10^6) = 0.5 seconds in between toggles, blinks with a frequency of 1 hz
//    This can be confirmed in the tracing as well (8 000 000 difference)
//
//    Commit your answers (bare6_1)
//
// 2. Now connect an oscilloscope to PC9, which is set to
//    output the MCO2.
//
//    Compute the value of SYSCLK based on the oscilloscope reading
//
//    ** My answer here **
//    Since the SYSCLK is sent directly to MCO2 we just need to measurement
//    Measured frequency: 4 Mhz
//    SYCLK = 4 * 4 = 16 Mhz.
//
//    What is the peak to peak (voltage) reading of the signal?
//
//    ** My answer here **
//    Delta Y: Around 3.5V
//
//    Make a folder called "pictures" in your git project.
//    Make a screen dump or photo of the oscilloscope output.
//    Save the the picture as "bare_6_16mhz_high_speed".
//
//    Commit your answers (bare6_2)
//
// 3. Now run the example in 48Mz, by commenting out line 56, and un-commenting
//    lines 58-63.
//`
//    What is the frequency of blinking?
//
//    ** My answer here **
//    8 000 000 / (48 * 10^6) = 1 / 6 in between toggles
//    The led blinks with a frequency of 3 hz
//
//    Now change the constant `OFFSET` so you get the same blinking frequency as in 1.
//    Test and validate that you got the desired behavior.
//
//    Commit your answers (bare6_3)
//
// 4. Repeat experiment 2
//
//    What is the frequency of MCO2 read by the oscilloscope?
//
//    ** My answer here **
//    12 Mhz, which is three times the reading for 16 Mhz
//    Getting an increase of 3 when we increase the clock frequency by a factor of three seems logical.
//
//    Compute the value of SYSCLK based on the oscilloscope reading.
//
//    ** My answer here **
//    It's 48 Mhz (4 * 12)
//
//    What is the peak to peak reading of the signal?
//
//    ** My answer here **
//    5.4V, this seems like a bit much.
//    We get 4.4V if we disconnect the ground. 
//
//    Make a screen dump or photo of the oscilloscope output.
//    Save the the picture as "bare_6_48mhz_high_speed".
//
//    Commit your answers (bare6_4)
//
// 5. In the `clock_out` function, the setup of registers is done through
//    setting bit-pattens manually, e.g.
//     rcc.cfgr
//        .modify(|_, w| unsafe { w.mco2().bits(0b00).mco2pre().bits(0b110) });
//
//    However based on the vendor SVD file the svd2rust API provides
//    a better abstraction, based on pattern enums and functions.
//
//    To view the API you can generate documentation for your crate:
//
//    > cargo doc --open
//
//    By searching for `mco2` you find the enumerations and functions.
//    So here
//       `w.mco2().bits{0b00}` is equivalent to
//       `w.mco2().sysclk()` and improves readability.
//
//    Replace all bit-patterns used in `clock_out` by the function name equivalents.
//    (alternatively, use the enum values.)
//
//    Test that the application still runs as before.
//
//    ** My answer here **
//    Fixed, works as before.
//
//    Commit your code (bare6_5)
//
// 6. Now reprogram the PC9 to be "Low Speed", and re-run at 48Mz.
//
//    Did the frequency change in comparison to assignment 5?
//
//    ** My answer here **
//    No, we still get 4 Mhz
//
//    What is the peak to peak reading of the signal (and why did it change)?
//
//    ** My answer here **
//    It dropped by 1v from 5.4 to 4.4ish.
//    As to the reason why, it's a bit unclear.
//    My best guess is that during the high_speed run we are updating so often
//    that we appproach the voltage of our power supply (5v), and since the speed is
//    lowered to low_speed, this will simply happen less often and we get futher away from
//    the voltage of our power supply.
//
//    Make a screen dump or photo of the oscilloscope output.
//    Save the the picture as "bare_6_48mhz_low_speed".
//
//    Commit your answers (bare6_6)
//
// 7. Try setting the clocks according to:
//
//    `rcc.cfgr.sysclk(64.mhz()).pclk1(64.mhz()).pclk2(64.mhz()).freeze()`;
//
//    Does the code compile?
//
//    ** My answer here **
//    The compiler doesn't know that the settings are incorrect, so yes.
//
//    What happens at run-time?
//
//    ** My answer here **
//    During runtime we get a panic
//
//    Try setting the clocks according to:
//
//    `rcc.cfgr.sysclk(84.mhz()).pclk1(42.mhz()).pclk2(64.mhz()).freeze();`
//
//    Does the code compile?
//
//    ** My answer here **
//    Yes, as there are no coding errors.
//
//    What happens at run-time?
//
//    ** My answer here **
//    It flashes the LED really fast, which is weird, according to STM32cubeMX
//    these settings hould be incompatible.
//    We can't get any measurements from the board, which was the expected outcome,
//    but it's a bit weird that there is no panic.
//
//    Is that a correct?
//
//    Optional: If you find it incorrect, file an issue to `stm32f4xx-hal` describing the problem.
//    (Remember always check already open issues, and add to existing if related.)
//
// 7. Discussion
//
//    In this exercise, you have learned to use the stm32f4xx-hal
//    to set the clock speed of your MCU.
//
//    You have also learned how you can monitor/validate MCU clock(s) on pin(s)
//    connected to an oscilloscope.
//
//    You have also learned how you can improve readability of your code
//    by leveraging the abstractions provided by the PAC.
//
//    As mentioned before the PACs are machine generated by `svd2rust`
//    from vendor provided System View Descriptions (SVDs).
//
//    The PACs provide low level peripheral access abstractions, while
//    the HALs provide higher level abstractions and functionality.
