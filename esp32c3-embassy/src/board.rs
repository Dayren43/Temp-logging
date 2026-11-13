//! Board definition for the **ESP32-C3-DevKitM-1** — the single source of truth
//! for pin assignments and peripheral bring-up. If the wiring changes, change it
//! here and nowhere else.

use esp_hal::clock::CpuClock;
use esp_hal::delay::Delay;
use esp_hal::gpio::{DriveMode, Flex, InputConfig, OutputConfig, Pull};
use esp_hal::interrupt::software::SoftwareInterruptControl;
use esp_hal::peripherals::{GPIO2, WIFI};
use esp_hal::rng::Rng;
use esp_hal::timer::timg::TimerGroup;
use sensor_driver::Dht22;

/// DHT22 driver bound to this board's data pin.
pub type Dht = Dht22<Flex<'static>, Delay>;

/// Ready-to-use resources produced by [`init`].
pub struct Board {
    /// DHT22 temperature/humidity sensor on GPIO2.
    pub dht: Dht,
    /// Wi-Fi radio peripheral (hand to `connect_wifi`).
    pub wifi: WIFI<'static>,
    /// Hardware RNG, used to seed the network stack.
    pub rng: Rng,
}

/// Bring up the board once at startup: max CPU clock, heap, the esp-rtos
/// scheduler kernel, and the peripherals this firmware uses.
pub fn init() -> Board {
    let p = esp_hal::init(esp_hal::Config::default().with_cpu_clock(CpuClock::max()));

    // esp-radio + smoltcp need a sizable heap.
    esp_alloc::heap_allocator!(#[esp_hal::ram(reclaimed)] size: 64 * 1024);
    esp_alloc::heap_allocator!(size: 36 * 1024);

    // esp-rtos is the scheduler kernel; Embassy + esp-radio run on top of it.
    let timg0 = TimerGroup::new(p.TIMG0);
    let sw_int = SoftwareInterruptControl::new(p.SW_INTERRUPT);
    esp_rtos::start(timg0.timer0, sw_int.software_interrupt0);

    Board {
        dht: dht22(p.GPIO2),
        wifi: p.WIFI,
        rng: Rng::new(),
    }
}

/// DHT22 single-wire bus on **GPIO2**: open-drain output + internal pull-up so
/// the one pin can both drive the line low and read it. (An external ~10k
/// pull-up is still recommended for reliable timing.)
fn dht22(gpio2: GPIO2<'static>) -> Dht {
    let mut pin = Flex::new(gpio2);
    pin.apply_input_config(&InputConfig::default().with_pull(Pull::Up));
    pin.apply_output_config(
        &OutputConfig::default()
            .with_drive_mode(DriveMode::OpenDrain)
            .with_pull(Pull::Up),
    );
    pin.set_input_enable(true);
    pin.set_output_enable(true);
    pin.set_high(); // idle high
    Dht22::new(pin, Delay::new())
}
