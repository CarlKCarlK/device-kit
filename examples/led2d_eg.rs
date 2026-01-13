#![no_std]
#![no_main]
#![allow(clippy::future_not_send, reason = "single-threaded")]

use core::{convert::Infallible, future};

use defmt::info;
use defmt_rtt as _;
use device_kit::led2d::rgb8_to_rgb888;
use device_kit::led2d::layout::LedLayout;
use device_kit::{Result, led2d};
use embassy_executor::Spawner;
use embassy_rp::init;
use panic_probe as _;
use smart_leds::colors;

// Two 12x4 panels stacked vertically for a 12x8 display.
const LED_LAYOUT_12X4: LedLayout<48, 12, 4> = LedLayout::serpentine_column_major();
const LED_LAYOUT_12X8: LedLayout<96, 12, 8> = LED_LAYOUT_12X4.concat_v(LED_LAYOUT_12X4);

led2d! {
    pub Led12x8Eg,
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

    let led12x8_eg = Led12x8Eg::new(p.PIN_4, p.PIO0, p.DMA_CH0, spawner)?;

    let mut led12x8_eg_frame = Led12x8EgFrame::new();

    let red = rgb8_to_rgb888(colors::RED);
    let green = rgb8_to_rgb888(colors::GREEN);

    let frame_top_left = Led12x8EgFrame::TOP_LEFT;
    let frame_size = Led12x8EgFrame::SIZE;

    use embedded_graphics::{
        Drawable,
        prelude::Primitive,
        primitives::{Circle, PrimitiveStyle, Rectangle},
    };

    Rectangle::new(frame_top_left, frame_size)
        .into_styled(PrimitiveStyle::with_stroke(red, 1))
        .draw(&mut led12x8_eg_frame)?;

    const CIRCLE_DIAMETER: u32 = 6;
    assert!(CIRCLE_DIAMETER <= Led12x8Eg::WIDTH as u32);
    assert!(CIRCLE_DIAMETER <= Led12x8Eg::HEIGHT as u32);

    let circle_top_left = embedded_graphics::prelude::Point::new(
        (Led12x8Eg::WIDTH as i32 - CIRCLE_DIAMETER as i32) / 2,
        (Led12x8Eg::HEIGHT as i32 - CIRCLE_DIAMETER as i32) / 2,
    );

    Circle::new(circle_top_left, CIRCLE_DIAMETER)
        .into_styled(PrimitiveStyle::with_stroke(green, 1))
        .draw(&mut led12x8_eg_frame)?;

    led12x8_eg.write_frame(led12x8_eg_frame).await?;

    future::pending().await
}
