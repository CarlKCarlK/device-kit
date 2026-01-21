#![no_std]
#![no_main]
#![cfg(not(feature = "host"))]

use core::{convert::Infallible, panic};

use device_kit::{
    Result,
    led_strip::{Current, Frame1d, colors, led_strip},
};
use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use {defmt_rtt as _, panic_probe as _};

led_strip! {
    LedStrip8 {
        pin: PIN_0,
        len: 8,
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

    let led_strip8 = LedStrip8::new(p.PIN_0, p.PIO0, p.DMA_CH0, spawner)?;

    // Create two frames with alternating blue and white dots
    let palette = [colors::BLUE, colors::GRAY];
    let mut frame0 = Frame1d::new();
    let mut frame1 = frame0.clone();
    for pixel_index in 0..frame0.len() {
        frame0[pixel_index] = palette[pixel_index % palette.len()];
        frame1[pixel_index] = palette[(pixel_index + 1) % palette.len()];
    }

    // Animate the frames in a loop
    loop {
        led_strip8.write_frame(frame0).await?;
        Timer::after(Duration::from_millis(150)).await;
        led_strip8.write_frame(frame1).await?;
        Timer::after(Duration::from_millis(150)).await;
    }
}
