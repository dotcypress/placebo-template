#![no_std]
#![no_main]
#![deny(warnings)]

extern crate cortex_m_rt as rt;
extern crate panic_halt;
extern crate stm32g0xx_hal as hal;

use hal::{gpio::*, prelude::*, serial, stm32, timer::*};
use ushell::{autocomplete, history::LRUHistory, UShell};

mod shell;

#[rtic::app(device = hal::stm32, peripherals = true)]
mod app {
    use super::*;

    #[local]
    struct Local {
        led: gpiob::PB3<Output<OpenDrain>>,
        shell: shell::Shell,
    }

    #[shared]
    struct Shared {
        timer: Timer<stm32::TIM2>,
    }

    #[init]
    fn init(ctx: init::Context) -> (Shared, Local, init::Monotonics) {
        let mut rcc = ctx.device.RCC.constrain();

        let port_a = ctx.device.GPIOA.split(&mut rcc);
        let port_b = ctx.device.GPIOB.split(&mut rcc);

        let led = port_b.pb3.into();

        let mut timer = ctx.device.TIM2.timer(&mut rcc);
        timer.start(250.millis());
        timer.listen();

        let uart_cfg = serial::BasicConfig::default().baudrate(115_200.bps());
        let mut uart = ctx
            .device
            .USART2
            .usart((port_a.pa2, port_a.pa3), uart_cfg, &mut rcc)
            .expect("Failed to init serial port");
        uart.listen(serial::Event::Rxne);

        let shell = UShell::new(
            uart,
            autocomplete::StaticAutocomplete(shell::AUTOCOMPLETE),
            LRUHistory::default(),
        );

        (Shared { timer }, Local { led, shell }, init::Monotonics())
    }

    #[task(binds = USART2, local = [shell], shared = [timer])]
    fn uart_rx(ctx: uart_rx::Context) {
        let mut env = ctx.shared;
        ctx.local.shell.spin(&mut env).ok();
    }

    #[task(binds = TIM2, local = [led], shared = [timer])]
    fn timer_tick(mut ctx: timer_tick::Context) {
        ctx.local.led.toggle().ok();
        ctx.shared.timer.lock(|timer| timer.clear_irq());
    }
}
