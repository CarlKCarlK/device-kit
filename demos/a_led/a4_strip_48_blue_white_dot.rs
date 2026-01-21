#![no_std]
#![no_main]
#![cfg(not(feature = "host"))]

use core::{convert::Infallible, future, panic};

use device_kit::{Result, led_strip::{Current, Frame1d, colors, led_strip}};
use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use {defmt_rtt as _, panic_probe as _};

led_strip! {
    LedStrip48 {
        pin: PIN_4,
        len: 48,
        max_current: Current::Milliamps(50),
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) -> ! {
    let err = inner_main(spawner).await.unwrap_err();
    panic!("{err}");
}

async fn inner_main(spawner: Spawner) -> Result<Infallible> {
    let p = embassy_rp::init(Default::default());

    let led_strip48 = LedStrip48::new(p.PIN_4, p.PIO0, p.DMA_CH0, spawner)?;

    for dot_index in (0..LedStrip48::LEN).cycle() {
        let mut frame1d = Frame1d::filled(colors::BLUE);
        frame1d[dot_index] = colors::WHITE;
        led_strip48.write_frame(frame1d).await?;
        Timer::after(Duration::from_millis(150)).await;
    }

    future::pending().await
}
