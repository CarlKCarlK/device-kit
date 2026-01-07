//! Example of what the [`led_strip!`](crate::led_strip!) macro generates.
//!
//! This module demonstrates the exact struct shape, associated constants, and constructor signature
//! produced by the [`led_strip!`](crate::led_strip!) macro. It uses the macro to generate a real struct
//! so you can inspect the generated API in documentation. All methods visible on this struct are
//! available on any strip created with the [`led_strip!`](crate::led_strip!) macro.
//!
//! # Configuration (Sample Values)
//!
//! This example uses:
//! - **`pin: PIN_3`** — GPIO pin (sample; use any available pin)
//! - **`len: 48`** — 48 LEDs (sample; adjust for your strip size)
//! - **`pio: PIO0` (default)** — PIO block (sample; switch to PIO1 if needed)
//! - **`dma: DMA_CH0` (default)** — DMA channel (sample; pick any available channel)
//!
//! See [`led_strip!`](crate::led_strip!) macro documentation for all available options.
//!
//! # Example Usage
//!
//! [`LedStripGenerated`] is equivalent to writing this macro invocation:
//!
//! ```ignore
//! led_strip! {
//!     LedStripGenerated {
//!         pin: PIN_3,       // ← Sample pin
//!         len: 48,          // ← Sample size
//!         // pio and dma use defaults; customize as needed
//!     }
//! }
//! ```
//!
//! After creation, all these methods are available (sample calls):
//!
//! ```ignore
//! let strip = LedStripGenerated::new(p.PIN_3, p.PIO0, p.DMA_CH0, spawner)?;
//!
//! // Write a single frame
//! let mut frame = Frame::filled(colors::RED);
//! frame[0] = colors::BLUE;  // Deref to [Rgb; N] for pixel access
//! strip.write_frame(&frame).await?;
//!
//! // Animate frames
//! strip.animate([
//!     (Frame::filled(colors::RED), Duration::from_millis(500)),
//!     (Frame::filled(colors::GREEN), Duration::from_millis(500)),
//! ]).await?;
//! ```
//!
//! # Generated Members
//!
//! - `const LEN: usize = 48` — The number of LEDs in the strip (changes based on `len` parameter)
//! - `const MAX_BRIGHTNESS: u8` — Maximum brightness (limited by power budget, see [`Current`](crate::led_strip::Current))
//! - `async fn new(pin, pio, dma, spawner) -> Result<Self>` — Constructor that sets up the LED strip

use crate::led_strip;

led_strip! {
    LedStripGenerated {
        pin: PIN_3,
        len: 48,
    }
}
