//! WiFi auto-provisioning demo with LED display showing Unix time.

#![no_std]
#![no_main]
#![cfg(feature = "wifi")]
#![allow(clippy::future_not_send, reason = "single-threaded")]

use core::{convert::Infallible, panic};
use defmt::warn;
use device_kit::{
    Result,
    button::PressedTo,
    flash_array::{FlashArray, FlashArrayStatic},
    led_strip::colors,
    led2d,
    led2d::{Led2dFont, layout::LedLayout},
    wifi_auto::{WifiAuto, WifiAutoEvent},
};
use embassy_executor::Spawner;
use embassy_net::{
    Stack,
    dns::DnsQueryType,
    udp::{PacketMetadata, UdpSocket},
};
use embassy_time::{Duration, Timer};
use {defmt_rtt as _, panic_probe as _};

// Set up LED layout for 8x12 panel (12x8 panel rotated 90 degrees clockwise)
const LED_LAYOUT_12X4: LedLayout<48, 12, 4> = LedLayout::serpentine_column_major();
const LED_LAYOUT_12X8: LedLayout<96, 12, 8> = LED_LAYOUT_12X4.combine_v(LED_LAYOUT_12X4);
const LED_LAYOUT_8X12: LedLayout<96, 8, 12> = LED_LAYOUT_12X8.rotate_cw();

// Color palette for display
const COLORS: &[smart_leds::RGB8] = &[
    colors::YELLOW,
    colors::LIME,
    colors::CYAN,
    colors::RED,
    colors::WHITE,
];

// We can't use default PIO0/DMA_CH0 for the LED display because that's used by WiFi.
led2d! {
    Led8x12 {
        pin: PIN_4,
        pio: PIO1,
        dma: DMA_CH1,
        led_layout: LED_LAYOUT_8X12,
        font: Led2dFont::Font4x6Trim,
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) -> ! {
    let err = inner_main(spawner).await.unwrap_err();
    panic!("{err}");
}

async fn inner_main(spawner: Spawner) -> Result<Infallible> {
    let p = embassy_rp::init(Default::default());

    // Flash stores WiFi credentials.
    static FLASH_STATIC: FlashArrayStatic = FlashArray::<1>::new_static();
    let [wifi_credentials_flash_block] = FlashArray::new(&FLASH_STATIC, p.FLASH)?;

    // Initialize WifiAuto. Pico Wifi uses internal pins.
    // A button is used to force reconfiguration via captive portal.
    let wifi_auto = WifiAuto::new(
        p.PIN_23,  // CYW43 power
        p.PIN_25,  // CYW43 chip select
        p.PIO0,    // CYW43 PIO interface (required)
        p.PIN_24,  // CYW43 clock
        p.PIN_29,  // CYW43 data
        p.DMA_CH0, // CYW43 DMA (required)
        wifi_credentials_flash_block,
        p.PIN_13, // Button for forced reconfiguration
        PressedTo::Ground,
        "PicoTime", // Captive-portal SSID
        [],         // Any custom fields
        spawner,
    )?;

    let led8x12 = Led8x12::new(p.PIN_4, p.PIO1, p.DMA_CH1, spawner)?;

    // Try to connect. Will launch captive portal if needed.
    // Returns network stack and button.
    //
    // (Reference here lets move closure capture led8x12 without taking ownership)
    let led8x12_ref = &led8x12;
    let (stack, _button) = wifi_auto
        .connect(spawner, move |event| async move {
            match event {
                WifiAutoEvent::CaptivePortalReady => {
                    led8x12_ref.write_text("CO\nNN", COLORS).await.ok();
                }
                WifiAutoEvent::Connecting { .. } => {
                    led8x12_ref.write_text("..\n..", COLORS).await.ok();
                }
                WifiAutoEvent::Connected => {
                    led8x12_ref.write_text("DO\nNE", COLORS).await.ok();
                }
                WifiAutoEvent::ConnectionFailed => {
                    led8x12_ref.write_text("FA\nIL", COLORS).await.ok();
                }
            }
        })
        .await?;

    // Show initial state with dashes until time arrives
    led8x12.write_text("--\n--", COLORS).await?;

    // Now use the network stack to fetch NTP time once per minute
    // and display the last 4 digits of the Unix timestamp.
    loop {
        match fetch_ntp_time(stack).await {
            Ok(unix_seconds) => {
                // Get last 4 digits of unix timestamp
                let last_4_digits = (unix_seconds % 10000) as u16;
                let time_str = format_4_digits_with_newline(last_4_digits);
                led8x12.write_text(&time_str, COLORS).await?;
            }
            Err(msg) => {
                warn!("NTP fetch failed: {}", msg);
                led8x12.write_text("--\n--", COLORS).await?;
            }
        }

        Timer::after(Duration::from_secs(60)).await;
    }
}

fn format_4_digits_with_newline(num: u16) -> heapless::String<6> {
    use core::fmt::Write;
    let mut s = heapless::String::new();
    let d1 = (num / 1000) % 10;
    let d2 = (num / 100) % 10;
    let d3 = (num / 10) % 10;
    let d4 = num % 10;
    write!(&mut s, "{}{}\n{}{}", d1, d2, d3, d4).unwrap();
    s
}

async fn fetch_ntp_time(stack: &Stack<'static>) -> core::result::Result<i64, &'static str> {
    const NTP_SERVER: &str = "pool.ntp.org";
    const NTP_PORT: u16 = 123;

    // DNS lookup
    let dns_result = stack
        .dns_query(NTP_SERVER, DnsQueryType::A)
        .await
        .map_err(|_| "DNS lookup failed")?;
    let server_addr = dns_result.first().ok_or("No DNS results")?;

    // Create UDP socket
    let mut rx_meta = [PacketMetadata::EMPTY; 1];
    let mut rx_buffer = [0; 128];
    let mut tx_meta = [PacketMetadata::EMPTY; 1];
    let mut tx_buffer = [0; 128];
    let mut socket = UdpSocket::new(
        *stack,
        &mut rx_meta,
        &mut rx_buffer,
        &mut tx_meta,
        &mut tx_buffer,
    );

    socket.bind(0).map_err(|_| "Socket bind failed")?;

    // Build NTP request (48 bytes, version 3, client mode)
    let mut ntp_request = [0u8; 48];
    ntp_request[0] = 0x1B; // LI=0, VN=3, Mode=3 (client)

    // Send request
    socket
        .send_to(&ntp_request, (*server_addr, NTP_PORT))
        .await
        .map_err(|_| "NTP send failed")?;

    // Receive response with timeout
    let mut response = [0u8; 48];
    embassy_time::with_timeout(Duration::from_secs(5), socket.recv_from(&mut response))
        .await
        .map_err(|_| "NTP receive timeout")?
        .map_err(|_| "NTP receive failed")?;

    // Extract transmit timestamp from response (bytes 40-43)
    let ntp_seconds = u32::from_be_bytes([response[40], response[41], response[42], response[43]]);

    // Convert NTP time (seconds since 1900) to Unix time (seconds since 1970)
    const NTP_TO_UNIX_OFFSET: i64 = 2_208_988_800;
    let unix_seconds = (ntp_seconds as i64) - NTP_TO_UNIX_OFFSET;

    Ok(unix_seconds)
}
