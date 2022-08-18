#![no_std]
#![no_main]
#![deny(warnings)]

extern crate cortex_m;
extern crate cortex_m_rt as rt;
extern crate panic_halt;
extern crate rtic;
extern crate stm32g0xx_hal as hal;

use defmt_rtt as _;

use core::fmt::Write;
use hal::gpio::{gpiob::*, *};
use hal::prelude::*;
use hal::serial::*;
use hal::stm32;
use hal::timer::*;

#[rtic::app(device = hal::stm32, peripherals = true)]
mod app {
    use super::*;

    #[shared]
    struct Shared {}

    #[local]
    struct Local {
        led: PB3<Output<OpenDrain>>,
        timer: Timer<stm32::TIM2>,
        uart: Serial<stm32::USART2, BasicConfig>,
    }

    #[init]
    fn init(ctx: init::Context) -> (Shared, Local, init::Monotonics) {
        defmt::info!("init");
        let mut rcc = ctx.device.RCC.constrain();

        let port_a = ctx.device.GPIOA.split(&mut rcc);
        let port_b = ctx.device.GPIOB.split(&mut rcc);

        let mut timer = ctx.device.TIM2.timer(&mut rcc);
        timer.start(250.millis());
        timer.listen();

        let led = port_b.pb3.into_open_drain_output();

        let uart_cfg = BasicConfig::default().baudrate(115_200.bps());
        let mut uart = ctx
            .device
            .USART2
            .usart((port_a.pa2, port_a.pa3), uart_cfg, &mut rcc)
            .expect("Failed to init serial port");

        uart.write_str("hello\r\n").ok();

        defmt::info!("init completed");
        (Shared {}, Local { timer, led, uart }, init::Monotonics())
    }

    #[task(binds = TIM2, local = [timer, led, uart])]
    fn timer_tick(ctx: timer_tick::Context) {
        let timer_tick::LocalResources { led, timer, uart } = ctx.local;

        led.toggle().ok();

        if led.is_set_high().unwrap_or_default() {
            defmt::info!("tick");
            uart.write_str("tick\r\n").ok();
        } else {
            defmt::info!("tock");
            uart.write_str("tock\r\n").ok();
        }

        timer.clear_irq();
    }

    #[idle]
    fn idle(_: idle::Context) -> ! {
        loop {
            cortex_m::asm::nop();
        }
    }
}
