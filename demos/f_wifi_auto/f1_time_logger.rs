//! Simple WiFi auto-provisioning demo that fetches time from NTP server every minute.
//!
//! This demo shows basic `WifiAuto` usage: connecting to WiFi through captive portal
//! provisioning, then querying an NTP (Network Time Protocol) server to get the current
//! time. It logs the Unix timestamp every minute.

#![no_std]
#![no_main]
#![cfg(feature = "wifi")]
#![allow(clippy::future_not_send, reason = "single-threaded")]

use core::{convert::Infallible, panic};
use defmt::{info, warn};
use device_kit::{
    Result,
    button::PressedTo,
    flash_array::{FlashArray, FlashArrayStatic},
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

const NTP_SERVER: &str = "pool.ntp.org";
const NTP_PORT: u16 = 123;

#[embassy_executor::main]
async fn main(spawner: Spawner) -> ! {
    let err = inner_main(spawner).await.unwrap_err();
    panic!("{err}");
}

async fn inner_main(spawner: Spawner) -> Result<Infallible> {
    info!("WiFi Auto NTP Demo - Starting");
    let p = embassy_rp::init(Default::default());

    // Set up flash storage for WiFi credentials
    static FLASH_STATIC: FlashArrayStatic = FlashArray::<1>::new_static();
    let [wifi_credentials_flash_block] = FlashArray::new(&FLASH_STATIC, p.FLASH)?;

    // Initialize WifiAuto - no custom fields needed for this simple demo
    let wifi_auto = WifiAuto::new(
        p.PIN_23,  // CYW43 power
        p.PIN_25,  // CYW43 chip select
        p.PIO0,    // CYW43 PIO interface
        p.PIN_24,  // CYW43 clock
        p.PIN_29,  // CYW43 data
        p.DMA_CH0, // CYW43 DMA
        wifi_credentials_flash_block,
        p.PIN_13, // Button for forced reconfiguration
        PressedTo::Ground,
        "PicoDemo", // Captive-portal SSID
        [],         // No custom fields
        spawner,
    )?;

    // Connect with status logging
    let (stack, _button) = wifi_auto
        .connect(spawner, |event| async move {
            match event {
                WifiAutoEvent::CaptivePortalReady => {
                    info!("Captive portal ready - connect to 'PicoDemo' WiFi network");
                }
                WifiAutoEvent::Connecting {
                    try_index,
                    try_count,
                } => {
                    info!(
                        "Connecting to WiFi (attempt {}/{})",
                        try_index + 1,
                        try_count
                    );
                }
                WifiAutoEvent::Connected => {
                    info!("WiFi connected successfully!");
                }
                WifiAutoEvent::ConnectionFailed => {
                    info!("WiFi connection failed - device will reset");
                }
            }
        })
        .await?;

    info!("WiFi connected - fetching time from NTP server");

    // Fetch and log time every minute
    loop {
        match fetch_ntp_time(stack).await {
            Ok(unix_seconds) => {
                info!("Current time: {} seconds since Unix epoch", unix_seconds);
            }
            Err(msg) => {
                warn!("NTP fetch failed: {}", msg);
            }
        }

        Timer::after(Duration::from_secs(60)).await;
    }
}

/// Fetch current time from NTP server and return Unix timestamp.
async fn fetch_ntp_time(stack: &Stack<'static>) -> core::result::Result<i64, &'static str> {
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
