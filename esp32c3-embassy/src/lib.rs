#![no_std]

//! Shared firmware library for the ESP32-C3 sensor node.
//!
//! Holds the [`board`] definition (pins + bring-up) and the runtime helpers used
//! by both the `esp32c3-embassy` firmware and the `selftest` runner, so neither
//! binary has to repeat the wiring or the Wi-Fi/networking boilerplate.

pub mod board;

use core::cell::Cell;

use embassy_net::{Runner, Stack, StackResources};
use embassy_sync::blocking_mutex::Mutex as BlockingMutex;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use esp_hal::peripherals::WIFI;
use esp_hal::rng::Rng;
use esp_radio::wifi::sta::StationConfig;
use esp_radio::wifi::{Config, ControllerConfig, Interface, Interfaces, WifiController};
use sensor_driver::Reading;

/// Wi-Fi credentials, supplied at compile time so they never enter a committed
/// file: `SSID="..." PASSWORD="..." cargo build`.
pub const SSID: &str = env!("SSID");
pub const PASSWORD: &str = env!("PASSWORD");

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    log::error!("{}", info);
    loop {}
}

/// Allocate a `'static` from a `StaticCell`. Stable-Rust equivalent of
/// `static_cell::make_static!`.
macro_rules! mk_static {
    ($t:ty, $val:expr) => {{
        static STATIC_CELL: ::static_cell::StaticCell<$t> = ::static_cell::StaticCell::new();
        STATIC_CELL.uninit().write($val)
    }};
}

/// Cache of the latest sensor reading: the sampler writes it and the HTTP
/// handler reads it instantly, so requests never wait on a sensor conversion.
static CACHE: BlockingMutex<CriticalSectionRawMutex, Cell<Option<Reading>>> =
    BlockingMutex::new(Cell::new(None));

/// Store the latest reading in the cache.
pub fn cache_store(reading: Reading) {
    CACHE.lock(|c| c.set(Some(reading)));
}

/// Load the latest cached reading, if any.
pub fn cache_load() -> Option<Reading> {
    CACHE.lock(|c| c.get())
}

/// Configure Wi-Fi station mode with the given credentials.
pub fn connect_wifi(
    wifi: WIFI<'static>,
    ssid: &str,
    password: &str,
) -> (WifiController<'static>, Interfaces<'static>) {
    let station = Config::Station(
        StationConfig::default()
            .with_ssid(ssid)
            .with_password(password.into()),
    );
    esp_radio::wifi::new(wifi, ControllerConfig::default().with_initial_config(station))
        .expect("Wi-Fi init failed")
}

/// Build the DHCPv4 network stack over the station interface.
pub fn net_stack(
    interface: Interface<'static>,
    rng: Rng,
) -> (Stack<'static>, Runner<'static, Interface<'static>>) {
    let seed = ((rng.random() as u64) << 32) | rng.random() as u64;
    embassy_net::new(
        interface,
        embassy_net::Config::dhcpv4(Default::default()),
        mk_static!(StackResources<3>, StackResources::<3>::new()),
        seed,
    )
}

/// Drives the embassy-net stack. Spawn exactly once.
#[embassy_executor::task]
pub async fn net_task(mut runner: Runner<'static, Interface<'static>>) {
    runner.run().await
}
