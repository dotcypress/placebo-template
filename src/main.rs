#![no_std]
#![no_main]
#![deny(warnings)]

extern crate panic_halt;
extern crate rtic;
extern crate stm32c0xx_hal as hal;

use defmt_rtt as _;

use hal::gpio::{gpioa::*, *};
use hal::prelude::*;
use hal::stm32;
use hal::timer::*;

#[rtic::app(device = hal::stm32, peripherals = true)]
mod app {
    use super::*;

    #[shared]
    struct Shared {}

    #[local]
    struct Local {
        frame: usize,
        led: PA0<Output<OpenDrain>>,
        timer: Timer<stm32::TIM16>,
    }

    #[init]
    fn init(ctx: init::Context) -> (Shared, Local, init::Monotonics) {
        defmt::info!("init");

        let mut rcc = ctx.device.RCC.constrain();

        let port_a = ctx.device.GPIOA.split(&mut rcc);
        let led = port_a.pa0.into_open_drain_output_in_state(PinState::High);

        let mut timer = ctx.device.TIM16.timer(&mut rcc);
        timer.start(50.millis());
        timer.listen();

        defmt::info!("init completed");

        (
            Shared {},
            Local {
                timer,
                led,
                frame: 0,
            },
            init::Monotonics(),
        )
    }

    #[task(binds = TIM16, local = [timer, led, frame])]
    fn timer_tick(ctx: timer_tick::Context) {
        let timer_tick::LocalResources { timer, led, frame } = ctx.local;

        let mask = 0b1001;
        if *frame & mask == mask {
            led.set_low().ok();
        } else {
            led.set_high().ok();
        }

        *frame += 1;
        timer.clear_irq();
    }

    #[idle]
    fn idle(_: idle::Context) -> ! {
        loop {
            rtic::export::nop();
        }
    }
}
