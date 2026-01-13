#![cfg(feature = "host")]

// cmk000000 need to inverse the gamma?

// check-all: skip (host-only PNG generation)

use device_kit::led2d::Frame2d;
use embedded_graphics::{
    pixelcolor::Rgb888,
    prelude::*,
    primitives::{Circle, PrimitiveStyle, Rectangle},
};
use png::{BitDepth, ColorType, Encoder};
use smart_leds::colors;
use std::error::Error;
use std::fs::File;
use std::io::BufWriter;
use std::path::{Path, PathBuf};

fn main() -> Result<(), Box<dyn Error>> {
    let output_path = output_path_from_args();
    let frame = build_frame();
    const CELL_SIZE: u32 = 64;
    const LED_MARGIN: u32 = 8;
    write_panel_png(&frame, &output_path, CELL_SIZE, LED_MARGIN)?;
    println!("wrote PNG to {}", output_path.display());
    Ok(())
}

fn output_path_from_args() -> PathBuf {
    let mut args = std::env::args().skip(1);
    if let Some(path) = args.next() {
        return PathBuf::from(path);
    }
    PathBuf::from("led2d_graphics.png")
}

fn build_frame() -> Frame2d<12, 8> {
    const WIDTH: usize = 12;
    const HEIGHT: usize = 8;
    const DIAMETER: u32 = 6;

    let mut frame = Frame2d::<WIDTH, HEIGHT>::new();

    Rectangle::new(
        Frame2d::<WIDTH, HEIGHT>::TOP_LEFT,
        Frame2d::<WIDTH, HEIGHT>::SIZE,
    )
    .into_styled(PrimitiveStyle::with_stroke(Rgb888::RED, 1))
    .draw(&mut frame)
    .expect("rectangle draw must succeed");

    frame[0][0] = colors::CYAN;

    let circle_top_left = centered_top_left(WIDTH, HEIGHT, DIAMETER as usize);
    Circle::new(circle_top_left, DIAMETER)
        .into_styled(PrimitiveStyle::with_stroke(Rgb888::GREEN, 1))
        .draw(&mut frame)
        .expect("circle draw must succeed");

    frame
}

fn centered_top_left(width: usize, height: usize, size: usize) -> Point {
    assert!(size <= width, "size must fit within width");
    assert!(size <= height, "size must fit within height");
    Point::new(((width - size) / 2) as i32, ((height - size) / 2) as i32)
}

fn write_panel_png<const W: usize, const H: usize>(
    frame: &Frame2d<W, H>,
    output_path: &Path,
    cell_size: u32,
    led_margin: u32,
) -> Result<(), Box<dyn Error>> {
    let (width, height, pixels) = panel_pixels(frame, cell_size, led_margin);
    if let Some(parent) = output_path.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent)?;
        }
    }

    let file = File::create(output_path)?;
    let mut encoder = Encoder::new(BufWriter::new(file), width, height);
    encoder.set_color(ColorType::Rgb);
    encoder.set_depth(BitDepth::Eight);
    let mut writer = encoder.write_header()?;
    writer.write_image_data(&pixels)?;
    Ok(())
}

fn panel_pixels<const W: usize, const H: usize>(
    frame: &Frame2d<W, H>,
    cell_size: u32,
    led_margin: u32,
) -> (u32, u32, Vec<u8>) {
    assert!(cell_size > 0, "cell_size must be positive");
    assert!(
        led_margin < cell_size / 2,
        "led_margin must fit inside cell"
    );
    let led_radius = (cell_size - (led_margin * 2)) / 2;
    assert!(led_radius > 0, "led_radius must be positive");
    let fade_width = led_radius / 3;
    assert!(fade_width > 0, "fade_width must be positive");

    let border = led_radius;
    assert!(border > 0, "border must be positive");
    let width = (W as u32) * cell_size + border * 2;
    let height = (H as u32) * cell_size + border * 2;
    let mut bytes = vec![0u8; (width * height * 3) as usize];
    let center = (cell_size - 1) as i32 / 2;
    let led_radius_f = led_radius as f32;
    let inner_radius_f = (led_radius - fade_width) as f32;
    let radius_sq = (led_radius as i32) * (led_radius as i32);

    for row_index in 0..H {
        for column_index in 0..W {
            let pixel = frame.0[row_index][column_index];
            let cell_origin_x = (column_index as u32) * cell_size;
            let cell_origin_y = (row_index as u32) * cell_size;

            for local_y in 0..cell_size {
                let delta_y = local_y as i32 - center;
                for local_x in 0..cell_size {
                    let delta_x = local_x as i32 - center;
                    let distance_sq = delta_x * delta_x + delta_y * delta_y;
                    if distance_sq <= radius_sq {
                        let distance = (distance_sq as f32).sqrt();
                        let intensity = if distance <= inner_radius_f {
                            1.0
                        } else {
                            let fade_span = led_radius_f - inner_radius_f;
                            (1.0 - (distance - inner_radius_f) / fade_span).max(0.0)
                        };
                        let x = border + cell_origin_x + local_x;
                        let y = border + cell_origin_y + local_y;
                        let pixel_index = ((y * width + x) * 3) as usize;
                        bytes[pixel_index] = (pixel.r as f32 * intensity).round() as u8;
                        bytes[pixel_index + 1] = (pixel.g as f32 * intensity).round() as u8;
                        bytes[pixel_index + 2] = (pixel.b as f32 * intensity).round() as u8;
                    }
                }
            }
        }
    }

    (width, height, bytes)
}
