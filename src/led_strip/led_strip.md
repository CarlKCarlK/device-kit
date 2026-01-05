<!-- markdownlint-disable MD041 -->

A device abstraction for NeoPixel-style (WS2812) LED strips.

You can set each LED to an individual color. This module treats a strip as a 1D line of lights. If your LED strip forms a grid, see [`Led2d`](crate::led2d::Led2d) for text rendering and graphics support.

## Example

Define a 48-LED strip and set every other LED to blue:

```rust
use device_kit::{Result, led_strip::{self, Frame, colors}};
use embassy_executor::Spawner;

led_strip! {
    LedStrip {
        pin: PIN_3,
        len: 48,
    }
}

async fn example(spawner: Spawner) -> Result<Infallible> {
    let p = embassy_rp::init(Default::default());

    let led_strip = LedStrip::new(p.PIN_3, p.PIO0, p.DMA_CH0, spawner)?;

    let mut frame = Frame::new();
    for pixel_index in (0..frame.len()).step_by(2) {
        frame[pixel_index] = colors::BLUE;
    }
    led_strip.write_frame(frame).await?;

    Ok(core::future::pending().await) // wait forever
}
```

See `examples/led_strip1.rs` for the complete program.

## Macro Configuration

In addition to specifying the GPIO `pin` and `len`, the `led_strip!` macro supports optional fields: `pio`, `dma`, `max_current`, `gamma`, and `max_frames`. See the Configuration section below for details.

## The `led_strip!` Macro

This macro generates a struct with a `new()` constructor that takes `(pin, pio, dma, spawner)`.
All [`LedStrip`] methods are available via `Deref`.

**Required fields:**

- `pin` — GPIO pin for LED data
- `len` — Number of LEDs

**Optional fields:**

- `pio` — PIO block (default: `PIO0`)
- `dma` — DMA channel (default: `DMA_CH0`)
- `max_current` — Current budget (default: `Current::Milliamps(250)`)
- `gamma` — Color curve (default: `Gamma::Gamma2_2`)
- `max_frames` — Animation buffer size (default: `16`)

## Current Limiting

The `max_current` field automatically scales brightness to stay within your power budget.

Each WS2812 LED is assumed to draw 60 mA at full brightness. If you specify:

```rust
max_current: Current::Milliamps(500),
len: 48,  // 48 × 60 = 2880 mA worst case
```

The generated `MAX_BRIGHTNESS` constant will limit all colors to ~17% of full brightness.

Use `Current::Unlimited` to disable limiting.

## Color Correction (Gamma)

The `gamma` field applies a color response curve to make colors look more natural:

- `Gamma::Linear` — No correction (raw values)
- `Gamma::Gamma2_2` — Standard sRGB curve (default, most natural-looking)

The curve is baked into a compile-time lookup table, so there's zero runtime cost.

## The `led_strips!` Macro (Advanced)

For **multiple strips sharing one PIO**, use `led_strips!` instead:

```rust
led_strips! {
    pio: PIO0,
    LedStripGroup {
        strip1: {
            pin: PIN_0,
            len: 8,
        },
        strip2: {
            pin: PIN_1,
            len: 16,
        },
    }
}

// Use the generated group constructor:
let (strip1, strip2) = LedStripGroup::new(
    p.PIO0, p.PIN_0, p.DMA_CH0, p.PIN_1, p.DMA_CH1, spawner
)?;
```

Most projects only need `led_strip!`. Use `led_strips!` only when you have multiple strips on different state machines.
