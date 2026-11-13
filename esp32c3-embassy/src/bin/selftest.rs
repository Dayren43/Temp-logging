#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]

//! On-hardware self-test runner.
//!
//! Runs a sequence of checks against the real board and prints PASS/FAIL over
//! the CP2102 serial, ending with a summary. Flash with credentials:
//!   SSID="..." PASSWORD="..." cargo build --release --bin selftest
//!   espflash flash --monitor target/.../release/selftest
//!
//! Tests: 1) DHT22 read   2) cache set/get   3) Wi-Fi associate+DHCP
//!        4) Wi-Fi reconnect (forced disconnect -> re-associate + re-DHCP)

use embassy_executor::Spawner;
use embassy_net::Stack;
use embassy_time::{Duration, with_timeout};
use log::{error, info};
use sensor_driver::Reading;

use esp32c3_embassy::board;
use esp32c3_embassy::{PASSWORD, SSID, cache_load, cache_store, connect_wifi, net_stack, net_task};

extern crate alloc;

esp_bootloader_esp_idf::esp_app_desc!();

#[esp_rtos::main]
async fn main(spawner: Spawner) -> ! {
    esp_println::logger::init_logger_from_env();

    let mut board = board::init();

    info!("==================== HARDWARE SELF-TEST ====================");
    let mut passed = 0u32;
    let total = 4u32;

    // ---- Test 1: DHT22 read returns a plausible value ----
    let sample = match critical_section::with(|_| board.dht.read()) {
        Ok(r)
            if (-40.0..80.0).contains(&r.temperature_c)
                && (0.0..=100.0).contains(&r.humidity_pct) =>
        {
            info!(
                "[1] sensor_read .......... PASS ({:.1} C, {:.1} %)",
                r.temperature_c, r.humidity_pct
            );
            passed += 1;
            Some(r)
        }
        Ok(r) => {
            error!("[1] sensor_read .......... FAIL (implausible: {:?})", r);
            None
        }
        Err(e) => {
            error!("[1] sensor_read .......... FAIL ({:?})", e);
            None
        }
    };

    // ---- Test 2: cache set/get round-trips ----
    let probe = sample.unwrap_or(Reading {
        temperature_c: 21.0,
        humidity_pct: 50.0,
    });
    cache_store(probe);
    if cache_load() == Some(probe) {
        info!("[2] cache_round_trip ..... PASS");
        passed += 1;
    } else {
        error!("[2] cache_round_trip ..... FAIL");
    }

    // ---- Wi-Fi bring-up (shared by tests 3 & 4) ----
    let (mut controller, interfaces) = connect_wifi(board.wifi, SSID, PASSWORD);
    let (stack, runner) = net_stack(interfaces.station, board.rng);
    spawner.spawn(net_task(runner).unwrap());

    // ---- Test 3: associate + DHCP within 30s ----
    let associated = match controller.connect_async().await {
        Ok(_) => wait_for_ip(stack, Duration::from_secs(30)).await,
        Err(e) => {
            error!("[3] wifi_associate ....... FAIL (connect error {:?})", e);
            false
        }
    };
    if associated {
        info!(
            "[3] wifi_associate ....... PASS (IP {})",
            stack.config_v4().unwrap().address
        );
        passed += 1;
    } else if controller.is_connected() {
        error!("[3] wifi_associate ....... FAIL (linked but no DHCP lease)");
    } else {
        error!("[3] wifi_associate ....... FAIL (no IP within 30s)");
    }

    // ---- Test 4: reconnect after a forced disconnect (router-reboot sim) ----
    if associated {
        info!("[4] forcing disconnect (simulating AP reboot)...");
        controller.disconnect_async().await.ok();
        // Wait (event-driven) for the link/lease to actually drop, so the
        // reconnect genuinely re-runs DHCP. Capped so a stuck link can't hang us.
        let _ = with_timeout(Duration::from_secs(5), stack.wait_config_down()).await;

        let recovered = match controller.connect_async().await {
            Ok(_) => wait_for_ip(stack, Duration::from_secs(30)).await,
            Err(e) => {
                error!("[4] wifi_reconnect ....... FAIL (reconnect error {:?})", e);
                false
            }
        };
        if recovered {
            info!(
                "[4] wifi_reconnect ....... PASS (re-acquired IP {})",
                stack.config_v4().unwrap().address
            );
            passed += 1;
        } else {
            error!("[4] wifi_reconnect ....... FAIL (no IP within 30s after reconnect)");
        }
    } else {
        error!("[4] wifi_reconnect ....... SKIP (associate failed)");
    }

    info!(
        "==================== RESULTS: {}/{} PASSED ====================",
        passed, total
    );
    if passed == total {
        info!("ALL TESTS PASSED");
    } else {
        error!("SOME TESTS FAILED");
    }

    // Tests finished — park forever (event-driven, no busy sleep).
    loop {
        core::future::pending::<()>().await;
    }
}

/// Wait (event-driven) for the stack to bring its DHCPv4 config up, bounded by
/// `timeout`. Returns true if an address was acquired in time.
async fn wait_for_ip(stack: Stack<'static>, timeout: Duration) -> bool {
    with_timeout(timeout, stack.wait_config_up()).await.is_ok()
}
