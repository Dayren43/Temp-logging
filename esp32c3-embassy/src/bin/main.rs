#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]

//! ESP32-C3 sensor firmware (esp-rtos + Embassy): DHT22 + Wi-Fi STA + HTTP pull.
//! Mirrors the original C firmware.
//! - `connection`  : robust Wi-Fi connect/reconnect loop (survives AP outages)
//! - `net_task`    : embassy-net runner (from the lib)
//! - `sample`      : reads the DHT22 every 2s into the shared cache
//! - `http_server` : serves the cached reading on GET /get (never blocks on the
//!   sensor), "Howdy" on GET /, 404 otherwise
//!
//! Build with credentials in the environment (never committed):
//!   SSID="..." PASSWORD="..." cargo build --release

use core::fmt::Write as _;

use embassy_executor::Spawner;
use embassy_net::Stack;
use embassy_net::tcp::TcpSocket;
use embassy_time::{Duration, Timer};
use embedded_io_async::Write as _;
use esp_radio::wifi::WifiController;
use log::{info, warn};

use esp32c3_embassy::board::{self, Dht};
use esp32c3_embassy::{PASSWORD, SSID, cache_load, cache_store, connect_wifi, net_stack, net_task};

extern crate alloc;

esp_bootloader_esp_idf::esp_app_desc!();

#[esp_rtos::main]
async fn main(spawner: Spawner) -> ! {
    esp_println::logger::init_logger_from_env();

    let board = board::init();
    info!("esp-rtos + Embassy started");

    let (controller, interfaces) = connect_wifi(board.wifi, SSID, PASSWORD);
    let (stack, runner) = net_stack(interfaces.station, board.rng);

    spawner.spawn(connection(controller).unwrap());
    spawner.spawn(net_task(runner).unwrap());
    spawner.spawn(sample(board.dht).unwrap());
    spawner.spawn(http_server(stack).unwrap());

    // Report IP / link transitions — driven by stack events, not polling.
    loop {
        stack.wait_config_up().await;
        if let Some(cfg) = stack.config_v4() {
            info!("Got IP: {} — try http://{}/get", cfg.address, cfg.address.address());
        }
        stack.wait_config_down().await;
        warn!("Link down (lost DHCP lease)");
    }
}

/// Read the DHT22 every 2s into the cache. The bit-bang is timing-sensitive, so
/// it runs in a critical section to keep esp-rtos's preemptive Wi-Fi tasks from
/// corrupting the protocol timing (~5ms, every 2s).
#[embassy_executor::task]
async fn sample(mut dht: Dht) {
    info!("sample task started");
    loop {
        match critical_section::with(|_| dht.read()) {
            Ok(r) => {
                cache_store(r);
                info!("[sample] {:.2} C  {:.2} %", r.temperature_c, r.humidity_pct);
            }
            Err(e) => warn!("[sample] read error: {:?}", e),
        }
        Timer::after(Duration::from_secs(2)).await;
    }
}

/// Tiny HTTP/1.0 responder mirroring the C server's pull model.
#[embassy_executor::task]
async fn http_server(stack: Stack<'static>) {
    let mut rx = [0u8; 1536];
    let mut tx = [0u8; 1536];
    info!("http server task started (port 80)");
    loop {
        let mut socket = TcpSocket::new(stack, &mut rx, &mut tx);
        socket.set_timeout(Some(Duration::from_secs(10)));

        if let Err(e) = socket.accept(80).await {
            warn!("accept error: {:?}", e);
            continue; // `accept` awaits the next connection; no busy spin
        }

        let mut req_buf = [0u8; 512];
        let n = match socket.read(&mut req_buf).await {
            Ok(0) => {
                socket.close();
                continue;
            }
            Ok(n) => n,
            Err(e) => {
                warn!("read error: {:?}", e);
                socket.abort();
                let _ = socket.flush().await;
                continue;
            }
        };
        let req = core::str::from_utf8(&req_buf[..n]).unwrap_or("");

        let (status, ctype, body) = route(req);

        let mut head: heapless::String<128> = heapless::String::new();
        let _ = write!(
            head,
            "HTTP/1.0 {}\r\nContent-Type: {}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
            status,
            ctype,
            body.len()
        );
        let _ = socket.write_all(head.as_bytes()).await;
        let _ = socket.write_all(body.as_bytes()).await;
        // `flush` returns only once the peer has acked the response, so the
        // client has the full body before we close and reuse the buffers.
        let _ = socket.flush().await;
        socket.close();
    }
}

/// Returns (status line, content-type, body). `/get` serves the cached reading.
fn route(req: &str) -> (&'static str, &'static str, heapless::String<96>) {
    let mut body: heapless::String<96> = heapless::String::new();
    if req.starts_with("GET /get") {
        match cache_load() {
            Some(r) => {
                let _ = write!(
                    body,
                    "{{\"Temp\":{:.1},\"Humid\":{:.1}}}",
                    r.temperature_c, r.humidity_pct
                );
                ("200 OK", "application/json", body)
            }
            None => {
                let _ = body.push_str("no sample yet");
                ("503 Service Unavailable", "text/plain", body)
            }
        }
    } else if req.starts_with("GET / ") || req.starts_with("GET /\r") {
        let _ = body.push_str("Howdy");
        ("200 OK", "text/plain", body)
    } else {
        let _ = body.push_str("Not found");
        ("404 Not Found", "text/plain", body)
    }
}

/// Robust connect/reconnect loop. On disconnect, waits 5s and reconnects.
#[embassy_executor::task]
async fn connection(mut controller: WifiController<'static>) {
    info!("connection task started");
    loop {
        match controller.connect_async().await {
            Ok(info) => {
                info!("Wi-Fi connected: {:?}", info);
                let why = controller.wait_for_disconnect_async().await;
                warn!("Wi-Fi disconnected ({:?}); reconnecting in 5s", why);
            }
            Err(e) => warn!("connect failed: {:?}; retrying in 5s", e),
        }
        Timer::after(Duration::from_millis(5000)).await;
    }
}
