//! Compile-only verification that led2d! macro visibility modifiers work correctly.
//!
//! This demonstrates that the new braced syntax supports all visibility levels.
//! Each test shows a different visibility level compiling successfully.
//!
//! Run via: `cargo check-all` (xtask compiles this for thumbv6m-none-eabi)

#![cfg(not(feature = "host"))]
#![no_std]
#![no_main]
#![allow(dead_code, reason = "Compile-time verification only")]

use defmt_rtt as _;
use device_kit::led_strip::{Current, Gamma};
use device_kit::led2d;
use device_kit::led2d::layout::LedLayout;
use embassy_executor::Spawner;
use panic_probe as _;

// Test with private visibility (no modifier) - this is the simplest test
const LED_LAYOUT: LedLayout<12, 4, 3> = LedLayout::serpentine_column_major();

led2d! {
    TestLed {
        pin: PIN_3,
        led_layout: LED_LAYOUT,
        font: Font3x4Trim,
        max_current: Current::Milliamps(100),
        gamma: Gamma::Linear,
    }
}

/// Compile-time verification that the braced syntax works and generates accessible types
fn test_led2d_braced_syntax_compilation() {
    // Test that the macro generates the expected types with the new braced syntax
    let _device_size = core::mem::size_of::<TestLed>();
    let _static_size = core::mem::size_of::<TestLedStatic>();

    // This compilation success demonstrates:
    // 1. Braced parameter syntax works (pin: PIN_3, etc.)
    // 2. All required parameters are parsed correctly
    // 3. Generated types are accessible with expected names
    // 4. Generated code compiles without errors
}

// Test public visibility using a second example
// Note: We test this separately in the module to avoid PIO resource conflicts
mod public_test {
    use super::*;

    const LED_LAYOUT_PUB: LedLayout<12, 4, 3> = LedLayout::serpentine_column_major();

    led2d! {
        pub PublicLed {
            pin: PIN_4,
            pio: PIO1,
            led_layout: LED_LAYOUT_PUB,
            font: Font3x4Trim,
            max_current: Current::Milliamps(100),
            gamma: Gamma::Linear,
        }
    }

    /// Test that pub visibility modifier works with braced syntax
    fn test_pub_visibility_compilation() {
        // This tests that `pub` visibility modifier is parsed and applied correctly
        let _device_size = core::mem::size_of::<PublicLed>();
        let _static_size = core::mem::size_of::<PublicLedStatic>();
    }
}

// Test crate visibility in a third module to avoid conflicts
mod crate_test {
    use super::*;

    const LED_LAYOUT_CRATE: LedLayout<8, 4, 2> = LedLayout::serpentine_column_major();

    led2d! {
        pub(crate) CrateLed {
            pin: PIN_5,
            pio: PIO0,
            dma: DMA_CH2,
            led_layout: LED_LAYOUT_CRATE,
            font: Font3x4Trim,
            max_current: Current::Milliamps(100),
            gamma: Gamma::Linear,
        }
    }

    /// Test that pub(crate) visibility modifier works with braced syntax
    fn test_crate_visibility_compilation() {
        // This tests that `pub(crate)` visibility modifier is parsed and applied correctly
        let _device_size = core::mem::size_of::<CrateLed>();
        let _static_size = core::mem::size_of::<CrateLedStatic>();
    }
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    // This main function exists only to satisfy the compiler.
    // The actual verification happens at compile time via the functions above.
    test_led2d_braced_syntax_compilation();
}

#[cfg(not(any(target_arch = "arm", target_arch = "riscv32", target_arch = "riscv64")))]
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo<'_>) -> ! {
    loop {}
}
