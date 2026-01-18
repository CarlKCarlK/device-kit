#![cfg(feature = "host")]
//cmk000000000000000 we can't have this file!

use core::ops::{Deref, DerefMut};
use embedded_graphics::prelude::RgbColor;

/// Predefined RGB color constants from the `smart_leds` crate.
#[doc(inline)]
pub use smart_leds::colors;

/// 8-bit-per-channel RGB color from `embedded_graphics`.
#[doc(inline)]
pub use embedded_graphics::pixelcolor::Rgb888;

/// RGB color type used by LED strip frames.
pub use smart_leds::RGB8;

/// Convert colors to [`RGB8`] for LED strip rendering.
///
/// # Example
///
/// ```rust,no_run
/// use device_kit::led_strip::{Rgb888, ToRgb8, RGB8};
///
/// let rgb8 = RGB8::new(16, 32, 48).to_rgb8();
/// let rgb888 = Rgb888::new(16, 32, 48);
/// let converted = rgb888.to_rgb8();
///
/// assert_eq!(rgb8, converted);
/// ```
pub trait ToRgb8 {
    /// Convert this color to [`RGB8`].
    #[must_use]
    fn to_rgb8(self) -> RGB8;
}

impl ToRgb8 for RGB8 {
    #[inline(always)]
    fn to_rgb8(self) -> RGB8 {
        self
    }
}

impl ToRgb8 for Rgb888 {
    #[inline(always)]
    fn to_rgb8(self) -> RGB8 {
        RGB8::new(self.r(), self.g(), self.b())
    }
}

/// Convert colors to [`Rgb888`] for embedded-graphics rendering.
///
/// # Example
///
/// ```rust,no_run
/// use device_kit::led_strip::{Rgb888, ToRgb888, RGB8};
///
/// let rgb8 = RGB8::new(16, 32, 48);
/// let rgb888 = rgb8.to_rgb888();
/// let already_rgb888 = Rgb888::new(16, 32, 48).to_rgb888();
///
/// assert_eq!(rgb888, already_rgb888);
/// ```
pub trait ToRgb888 {
    /// Convert this color to [`Rgb888`].
    #[must_use]
    fn to_rgb888(self) -> Rgb888;
}

impl ToRgb888 for RGB8 {
    #[inline(always)]
    fn to_rgb888(self) -> Rgb888 {
        Rgb888::new(self.r, self.g, self.b)
    }
}

impl ToRgb888 for Rgb888 {
    #[inline(always)]
    fn to_rgb888(self) -> Rgb888 {
        self
    }
}

/// Fixed-size 1D LED strip frame.
#[derive(Clone, Copy, Debug)]
pub struct Frame1d<const N: usize>(pub [RGB8; N]);

impl<const N: usize> Frame1d<N> {
    /// Number of LEDs in this frame.
    pub const LEN: usize = N;

    /// Create a new blank (all black) frame.
    #[must_use]
    pub const fn new() -> Self {
        Self([RGB8::new(0, 0, 0); N])
    }

    /// Create a frame filled with a single color.
    #[must_use]
    pub const fn filled(color: RGB8) -> Self {
        Self([color; N])
    }
}

impl<const N: usize> Deref for Frame1d<N> {
    type Target = [RGB8; N];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<const N: usize> DerefMut for Frame1d<N> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<const N: usize> From<[RGB8; N]> for Frame1d<N> {
    fn from(array: [RGB8; N]) -> Self {
        Self(array)
    }
}

impl<const N: usize> From<Frame1d<N>> for [RGB8; N] {
    fn from(frame: Frame1d<N>) -> Self {
        frame.0
    }
}

impl<const N: usize> Default for Frame1d<N> {
    fn default() -> Self {
        Self::new()
    }
}
