#![no_std]
#![no_main]
#![deny(warnings)]

extern crate cortex_m;
extern crate cortex_m_rt as rt;
extern crate panic_halt;
extern crate rtic;
extern crate stm32g0xx_hal as hal;

use defmt_rtt as _;

use hal::gpio::{gpiob::*, *};
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
        led: PB3<Output<OpenDrain>>,
        timer: Timer<stm32::TIM16>,
    }

    #[init]
    fn init(ctx: init::Context) -> (Shared, Local, init::Monotonics) {
        defmt::info!("init");

        let mut rcc = ctx.device.RCC.constrain();

        let port_b = ctx.device.GPIOB.split(&mut rcc);
        let led = port_b.pb3.into_open_drain_output_in_state(PinState::High);

        let mut timer = ctx.device.TIM16.timer(&mut rcc);
        timer.start(20.millis());
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

        let mask = 0b10001;
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
            cortex_m::asm::nop();
        }
    }
}
