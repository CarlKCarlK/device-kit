# Demos

## A1 – 8-LED strip, blue/gray alternating

Verifies wiring, PIO, DMA, and the LED strip device abstraction.
Demonstrates a frame as pure data using a palette-based pattern.

Run/flash (Pico 1 ARM):

```bash
cargo run --release --bin demo_a1_strip_8_blue_gray --target thumbv6m-none-eabi --features pico1,arm
```

## A2 – 8-LED strip, blue/white with moving black dot

Animates a single black dot over a blue/white background without using `animate`.
Shows frame updates in a tight loop inside the demo.

Run/flash (Pico 1 ARM):

```bash
cargo run --release --bin demo_a2_strip_8_blue_white_dot --target thumbv6m-none-eabi --features pico1,arm
```
