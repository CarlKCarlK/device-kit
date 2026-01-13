#![no_std]
#![no_main]
#![allow(clippy::future_not_send, reason = "single-threaded")]

use core::{convert::Infallible, future};

use defmt::info;
use defmt_rtt as _;
use device_kit::led2d::Frame2d;
use device_kit::led2d::layout::LedLayout;
use device_kit::{Result, led2d};
use embassy_executor::Spawner;
use embassy_rp::init;
use embedded_graphics::{
    Drawable,
    pixelcolor::Rgb888,
    prelude::Point,
    prelude::Primitive,
    prelude::RgbColor,
    primitives::{Circle, PrimitiveStyle, Rectangle},
};
use panic_probe as _;

// Two 12x4 panels stacked vertically for a 12x8 display.
const LED_LAYOUT_12X4: LedLayout<48, 12, 4> = LedLayout::serpentine_column_major();
const LED_LAYOUT_12X8: LedLayout<96, 12, 8> = LED_LAYOUT_12X4.concat_v(LED_LAYOUT_12X4);

led2d! {
    pub Led12x8,
    pin: PIN_4,
    width: 12,
    height: 8,
    led_layout: LED_LAYOUT_12X8,
    font: Font4x6Trim,
}

#[embassy_executor::main]
pub async fn main(spawner: Spawner) -> ! {
    let err = inner_main(spawner).await.unwrap_err();
    core::panic!("{err}");
}

async fn inner_main(spawner: Spawner) -> Result<Infallible> {
    info!("LED 2D Example: Rectangle border with centered circle on a 12x8 panel");
    let p = init(Default::default());

    let led12x8 = Led12x8::new(p.PIN_4, p.PIO0, p.DMA_CH0, spawner)?;

    let mut frame = Frame2d::new();

    Rectangle::new(Led12x8::TOP_LEFT, Led12x8::SIZE)
        .into_styled(PrimitiveStyle::with_stroke(Rgb888::RED, 1))
        .draw(&mut frame)?;

    const DIAMETER: u32 = 6;
    const TOP_LEFT: Point = centered_top_left(Led12x8::WIDTH, Led12x8::HEIGHT, DIAMETER);
    Circle::new(TOP_LEFT, DIAMETER)
        .into_styled(PrimitiveStyle::with_stroke(Rgb888::GREEN, 1))
        .draw(&mut frame)?;

    led12x8.write_frame(frame).await?;

    future::pending().await // Run forever
}

const fn centered_top_left(width: usize, height: usize, size: u32) -> Point {
    assert!(size <= width as u32); // compile-time check
    assert!(size <= height as u32); // compile-time check

    Point::new(
        (width as i32 - size as i32) / 2,
        (height as i32 - size as i32) / 2,
    )
}
