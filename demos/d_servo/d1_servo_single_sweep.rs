#![no_std]
#![no_main]
#![cfg(not(feature = "host"))]

use core::{convert::Infallible, panic};
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
    servo.set_degrees(90);
    Timer::after_millis(400).await;
    servo.set_degrees(180);
    Timer::after_millis(400).await;
    servo.center();
    Timer::after_millis(400).await;

    const STEP_DEGREES: u16 = 10;
    const MAX_DEGREES: u16 = 180;
    let mut degrees: u16 = 0;

    loop {
        button.wait_for_press().await;
        degrees = if degrees + STEP_DEGREES > MAX_DEGREES {
            0
        } else {
            degrees + STEP_DEGREES
        };
        servo.set_degrees(degrees);
    }
}
