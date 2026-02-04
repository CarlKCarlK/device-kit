# device-envoy

**Build Pico applications with LED panels, easy WiFi, and composable device abstractions.**

`device-envoy` explores application-level device abstractions in embedded Rust using the Embassy async framework. It focuses on building reusable, typed async APIs that hide timing, interrupts, channels, and shared state inside each device.

`device-envoy` sits above HALs and drivers, modeling devices as long-lived async tasks with typed APIs rather than pin-level control.

Currently targeting Raspberry Pi Pico 1 and Pico 2 (ARM cores). RISC-V core support exists but is not actively tested.

## Status

⚠️ **Alpha / Experimental** (version 0.0.2-alpha)

The API is actively evolving. Not recommended for production use, but excellent for experimentation and embedded Rust learning.

**Background:** See [How Rust & Embassy Shine on Embedded Devices](https://medium.com/@carlmkadie/how-rust-embassy-shine-on-embedded-devices-part-1-9f4911c92007) by Carl M. Kadie and Brad Gibson.

## Features

- **LED Panels & Strips** - NeoPixel-style (WS2812) LED arrays with 2D text rendering, animation, and embedded-graphics support
- **WiFi (Pico W)** - Connect to the Internet with automatic credentials management. On boot, opens a web form if WiFi credentials aren't saved, then connects seamlessly to stored networks. Requires Pico W; WiFi is not supported on non-W boards.
- **Button Input** - Button handling with debouncing
- **Servo Control** - Servo positioning and animation
- **Flash Storage** - Type-safe, on-board persist storage
- **LCD Display** - Text display (HD44780)
- **IR Remote** - Remote control decoder (NEC protocol)
- **RFID Reader** - Card detection and reading (MFRC522)

## Examples & Demos

The project includes **examples** (single-device tests) in `examples/` and **demo applications** in `demos/` showing real-world integration patterns:

- **Basic LED Examples**: Simple on/off control with blinky pattern
- **LED Strip Examples**: Simple animations, color control, text rendering
- **LED Panel Examples**: 12×4, 12×8, and multi-panel configurations with graphics
- **Button Examples**: Debouncing and state handling
- **Servo Examples**: Position sweeps and animation playback
- **WiFi Examples**: WiFi setup, time sync, DNS
- **Flash Examples**: Configuration persistence and data reset

See the `examples/` and `demos/` directories for complete runnable code.

## Building & Running

### Prerequisites

```bash
# Add Rust targets for Pico boards
rustup target add thumbv6m-none-eabi           # Pico 1 (ARM)
rustup target add thumbv8m.main-none-eabihf    # Pico 2 (ARM)
```

### Quick Start

```bash
# Run examples using convenient aliases
cargo blinky                # Simple LED blinky (Pico 1)
cargo blinky-2              # Simple LED blinky (Pico 2)

cargo clock-lcd-w           # LCD clock with WiFi (Pico 1 WiFi)
cargo clock-lcd-2w          # LCD clock with WiFi (Pico 2 WiFi)

cargo clock-led12x4-w       # LED panel clock (Pico 1 WiFi)
cargo clock-led12x4-2w      # LED panel clock (Pico 2 WiFi)

# Check without running (faster builds)
cargo blinky-check          # Compile only
cargo clock-lcd-w-check     # Check Pico 1 WiFi version

# Build and check everything
cargo check-all
```

**Tools:**

- `just` - Optional command runner (install with `cargo install just` or your package manager). See `justfile` for commands.
- `xtask` - Project's custom automation tool (built-in, use via `cargo xtask --help`)

See `.cargo/config.toml` for all cargo aliases.

## Hardware Notes

### Standard Pinouts

Examples use conventional pin assignments for consistency:

- **PIN_0**: LED strip (8-pixel simple example)
- **PIN_1**: Single LED (blinky patterns) - Built-in LEDs are modeled as active-high (OnLevel::High) on all supported boards
- **PIN_3**: LED panel (12×4, 48 pixels)
- **PIN_4**: Extended LED panel (12×8, 96 pixels)
- **PIN_5**: Long LED strip (160 pixels, broadway/marquee effects)
- **PIN_6**: Large LED panel (16×16, 256 pixels)
- **PIN_13**: Button (active-low)
- **PIN_11, PIN_12**: Servo signals

## Testing

Host-side tests run on your development machine without hardware:

```bash
# Run host tests (unit + integration)
cargo test --no-default-features --features defmt,host

# Or run via xtask
cargo xtask check-docs  # Includes doc tests
```

Tests include:

- LED text rendering comparisons against reference images
- 2D LED matrix mapping algebra
- LED color space conversions

## License

Licensed under either:

- MIT license (see LICENSE-MIT file)
- Apache License, Version 2.0

at your option.
