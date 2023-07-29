//! examples/adc.rs
#![no_main]
#![no_std]

#[rtic::app(
    device = stm32h7xx_hal::stm32,
    peripherals = true,
)]
mod app {
    use log::info;
    // Includes a panic handler and optional logging facilities
    use libdaisy::logger;

    use libdaisy::adc::{Adc, ChannelConfig};
    use libdaisy::system;

    use libdaisy::prelude::*;
    use stm32h7xx_hal::stm32;
    use stm32h7xx_hal::timer::Timer;

    #[shared]
    struct Shared {}

    #[local]
    struct Local {
        adc: Adc,
        timer2: Timer<stm32::TIM2>,
    }

    #[init]
    fn init(ctx: init::Context) -> (Shared, Local, init::Monotonics) {
        logger::init();
        let mut system = system::System::init(ctx.core, ctx.device);
        system.timer2.set_freq(1.Hz());

        let daisy21 = system
            .gpio
            .daisy21
            .take()
            .expect("Failed to get pin 21!")
            .into_analog();

        let config = ChannelConfig::new(daisy21);
        system.adc.init_single(config);
        system.adc.start();

        info!("starting!...");

        (
            Shared {},
            Local {
                adc: system.adc,
                timer2: system.timer2,
            },
            init::Monotonics(),
        )
    }

    #[idle]
    fn idle(_cx: idle::Context) -> ! {
        loop {
            cortex_m::asm::nop();
        }
    }

    #[task(binds = TIM2, local = [timer2, adc])]
    fn timer_handler(ctx: timer_handler::Context) {
        ctx.local.timer2.clear_irq();
        info!("{} {}", ctx.local.adc.get(0), ctx.local.adc.get_float(0));
    }
}
