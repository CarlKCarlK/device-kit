//! Dual servo control example.
//! Moves two servos in opposite directions for 2 seconds.
//! Connect servos to GPIO 0 and GPIO 2.

#![no_std]
#![no_main]

use defmt::info;
use defmt_rtt as _;
use device_kit::servo;
use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use panic_probe as _;

#[embassy_executor::main]
pub async fn main(_spawner: Spawner) -> ! {
    let p = embassy_rp::init(Default::default());

    info!("Starting dual servo example");
    // cmk00 test on variables w/o the name
    // cmk00 test when slice is wrong one

    // Create servos on GPIO 0 and GPIO 2 (both even pins)
    // GPIO 0 → (0/2) % 8 = 0 → PWM_SLICE0
    // GPIO 2 → (2/2) % 8 = 1 → PWM_SLICE1
    let some_pin = p.PIN_0;
    let some_slice = p.PWM_SLICE1;
    let mut servo0 = servo! {
        pin: some_pin,
        slice: some_slice,
    };
    let mut servo2 = servo! {
        pin: p.PIN_2,
        slice: p.PWM_SLICE1,
    };

    info!("Moving servos in opposite directions for 2 seconds");

    let start = embassy_time::Instant::now();
    let duration = Duration::from_secs(2);

    loop {
        let elapsed = start.elapsed();
        if elapsed > duration {
            break;
        }

        // Move servos in opposite directions
        info!("Position: servo0=0°, servo2=180°");
        servo0.set_degrees(0);
        servo2.set_degrees(180);
        Timer::after_millis(500).await;

        // Move servos in opposite directions (swapped)
        info!("Position: servo0=180°, servo2=0°");
        servo0.set_degrees(180);
        servo2.set_degrees(0);
        Timer::after_millis(500).await;
    }

    info!("Done! Centering servos");
    servo0.center();
    servo2.center();

    Timer::after_millis(500).await;

    info!("Relaxing servos");
    servo0.disable();
    servo2.disable();

    Timer::after_secs(5).await;

    loop {
        info!("Sleeping");
        Timer::after_secs(5).await;
    }
}
