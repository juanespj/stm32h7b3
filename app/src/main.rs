#![no_std]
#![no_main]

use defmt::info;
use embassy_executor::Spawner;
use embassy_stm32::gpio::{Level, Output, Speed};
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::task]
async fn blinky(mut led: Output<'static>) -> ! {
    loop {
        info!("LED on");
        led.set_high();
        Timer::after_millis(500).await;

        info!("LED off");
        led.set_low();
        Timer::after_millis(500).await;
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) -> ! {
    let mut config = embassy_stm32::Config::default();

    config.rcc.supply_config = embassy_stm32::rcc::SupplyConfig::LDO;

    let p = embassy_stm32::init(config);

    info!("Application started from 0x08010000!");

    let led = Output::new(p.PG13, Level::Low, Speed::Low);
    spawner.spawn(blinky(led).unwrap());

    loop {
        Timer::after_millis(1000).await;
    }
}
