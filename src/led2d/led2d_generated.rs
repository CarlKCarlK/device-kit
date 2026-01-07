//! Example of what the [`led2d!`](crate::led2d!) macro generates.
//!
//! This module is documentation-only and demonstrates the exact struct shape, associated constants, and constructor signature
//! produced by the [`led2d!`](crate::led2d!) macro. See actual examples in `examples/` directory for compilable code.
//!
//! # Configuration (Sample Values)
//!
//! This example uses:
//! - **`pio: PIO0`** — PIO block (sample; switch to PIO1 if needed)
//! - **`pin: PIN_3`** — GPIO pin (sample; use any available pin)
//! - **`dma: DMA_CH0`** — DMA channel (sample; pick any available channel)
//! - **`width: 12, height: 4`** — 12×4 panel (48 LEDs; sample; adjust for your display size)
//! - **`led_layout: serpentine_row_major`** — Physical LED arrangement (sample)
//! - **`max_current: Unlimited` (default)** — Power budget
//! - **`gamma: Gamma2_2` (default)** — Color correction
//! - **`max_frames: 16` (default)** — Animation buffer
//! - **`font: Font3x4Trim`** — Text rendering font (sample)
//!
//! See [`led2d!`](crate::led2d!) macro documentation for all available options.
//!
//! # Example Macro Invocation
//!
//! The generated struct is created with a macro invocation like this:
//!
//! ```ignore
//! led2d! {
//!     Led2DGenerated,               // ← Struct name
//!     pio: PIO0,                    // ← Sample PIO
//!     pin: PIN_3,                   // ← Sample pin
//!     dma: DMA_CH0,                 // ← Sample DMA channel
//!     width: 12,                    // ← Sample width
//!     height: 4,                    // ← Sample height
//!     led_layout: serpentine_row_major,  // ← Sample layout
//!     max_current: Current::Unlimited,   // ← Default
//!     gamma: Gamma::Gamma2_2,       // ← Default
//!     max_frames: 16,               // ← Default
//!     font: Font3x4Trim,            // ← Sample font
//! }
//! ```
//!
//! # Example Usage (After Generation)
//!
//! After the macro generates your struct, these methods are available:
//!
//! ```ignore
//! let led2d = Led2DGenerated::new(p.PIN_3, p.PIO0, p.DMA_CH0, spawner)?;
//!
//! // Write text
//! use device_kit::led_strip::colors;
//! led2d.write_text("HI", &[colors::RED]).await?;
//!
//! // Use the graphics API with embedded_graphics
//! let mut frame = Led2DGenerated::new_frame();
//! // ... draw with embedded_graphics ...
//! led2d.write_frame(frame).await?;
//! ```
//!
//! # Generated Public API
//!
//! - `const WIDTH: u32 = 12` — Panel width (changes based on `width` parameter)
//! - `const HEIGHT: u32 = 4` — Panel height (changes based on `height` parameter)
//! - `const LEN: usize = 48` — Total number of LEDs (WIDTH × HEIGHT)
//! - `const MAX_BRIGHTNESS: u8` — Maximum brightness (limited by power budget)
//! - `async fn new(pin, pio, dma, spawner) -> Result<Self>` — Constructor that sets up the 2D display
//! - `async fn write_text(text, colors) -> Result<()>` — Render text
//! - `async fn write_frame(frame) -> Result<()>` — Display a graphics frame
//! - `fn new_frame() -> Frame` — Create a blank frame for drawing
