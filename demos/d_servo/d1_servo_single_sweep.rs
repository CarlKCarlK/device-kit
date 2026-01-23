#![no_std]
#![no_main]
#![cfg(not(feature = "host"))]

use core::{convert::Infallible, future, panic};
use device_kit::{
    Result,
    button::{Button, PressedTo},
    servo,
};
use embassy_executor::Spawner;
use embassy_time::Timer;
use {defmt::info, defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(spawner: Spawner) -> ! {
    let err = inner_main(spawner).await.unwrap_err();
    panic!("{err}");
}

async fn inner_main(_spawner: Spawner) -> Result<Infallible> {
    let p = embassy_rp::init(Default::default());

    info!("Starting servo demo (GPIO 11)");
    let mut servo = servo! {
        pin: p.PIN_11,
        slice: p.PWM_SLICE5, // 11 → (11/2) % 8 = 5
    };
    let mut button = Button::new(p.PIN_13, PressedTo::Ground);
    servo.set_degrees(0);
    Timer::after_millis(400).await;
    servo.set_degrees(180);
    Timer::after_millis(400).await;
    servo.center(); // cmk000 need "center"

    // Loop by 10 degrees. Include 180 degrees.
    for degree in (0..=180).step_by(10).cycle() {
        button.wait_for_press().await;
        servo.set_degrees(degree);
    }

    future::pending().await // Needed because compiler can't see that loop is infinite
}
