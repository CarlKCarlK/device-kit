#![allow(missing_docs)]
#![no_std]
#![no_main]
#![cfg(not(feature = "host"))]

use core::{convert::Infallible, future, panic};
use device_kit::{
    Result,
    flash_array::{FlashArray, FlashArrayStatic},
    led_strip::{Current, Gamma, colors},
    led2d,
    led2d::{Led2dFont, layout::LedLayout},
};
use embassy_executor::Spawner;
use {defmt_rtt as _, panic_probe as _};

/// Boot counter (newtype) that wraps at 10.
/// Stored with `postcard` (Serde).
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
struct BootCounter(u8);

impl BootCounter {
    const fn new(value: u8) -> Self {
        Self(value)
    }

    fn increment(self) -> Self {
        Self((self.0 + 1) % 10)
    }
}

// Set up LED layout for 12x8 panel.
const LED_LAYOUT_12X4: LedLayout<48, 12, 4> = LedLayout::serpentine_column_major();
const LED_LAYOUT_12X8_ROTATED: LedLayout<96, 8, 12> =
    LED_LAYOUT_12X4.combine_v(LED_LAYOUT_12X4).rotate_cw();

led2d! {
    Led12x8 {
        pio: PIO0,
        pin: PIN_4,
        dma: DMA_CH1,
        led_layout: LED_LAYOUT_12X8_ROTATED,
        max_current: Current::Milliamps(500),
        gamma: Gamma::Linear,
        max_frames: 32,
        font: Led2dFont::Font7x12Trim,
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) -> ! {
    let err = inner_main(spawner).await.unwrap_err();
    panic!("{err}");
}

async fn inner_main(spawner: Spawner) -> Result<Infallible> {
    let p = embassy_rp::init(Default::default());

    // Create a flash array with one element. Each element holds up to 3900 bytes
    // of postcard-serialized data (one 4 KB flash erase sector minus metadata).
    static FLASH_STATIC: FlashArrayStatic = FlashArray::<1>::new_static();
    let mut flash_array = FlashArray::<1>::new(&FLASH_STATIC, p.FLASH)?;

    // Read boot counter from flash then increment.
    // FlashArray includes a runtime type hash so values are only loaded
    // if the stored type matches the requested type; mismatches yield `None`.
    let boot_counter = flash_array[0]
        .load()?
        .unwrap_or(BootCounter::new(0)) // Default to 0 if not present
        .increment();

    // Write incremented counter back to flash.
    // This writes once per power-up (fine for a demo; don't write in a tight loop).
    // Flash is typically good for ~100K erase cycles per sector.
    flash_array[0].save(&boot_counter)?;

    // Display the counter on the LED panel (single digit 0-9)
    let led12x8 = Led12x8::new(p.PIN_4, p.PIO0, p.DMA_CH1, spawner)?;
    const DIGITS: [&str; 10] = ["0", "1", "2", "3", "4", "5", "6", "7", "8", "9"];
    led12x8
        .write_text(DIGITS[boot_counter.0 as usize], &[colors::RED])
        .await?;

    future::pending().await // Keep running
}

// Not shown:
// `FlashArray` supports array destructuring, which can be convenient:
// static FLASH3: FlashArrayStatic = FlashArray::<3>::new_static();
// let [mut a, mut b, mut c] = FlashArray::<3>::new(&FLASH3, p.FLASH)?;
