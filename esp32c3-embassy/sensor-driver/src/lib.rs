#![no_std]
//! Hardware-agnostic DHT22 / AM2302 driver.
//!
//! Generic over any pin that is both [`InputPin`] and [`OutputPin`] (wire it
//! open-drain with an external ~10k pull-up, exactly like the C version on
//! GPIO2) plus any [`DelayNs`] source. Nothing in here knows about the ESP32 —
//! swap the HAL and the driver is unchanged.

use embedded_hal::delay::DelayNs;
use embedded_hal::digital::{InputPin, OutputPin};

/// A decoded temperature + humidity reading.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Reading {
    pub temperature_c: f32,
    pub humidity_pct: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    /// The sensor did not drive the line within the expected window.
    Timeout,
    /// Checksum byte did not match the payload.
    Checksum,
    /// Underlying GPIO error.
    Pin,
}

pub struct Dht22<P, D> {
    pin: P,
    delay: D,
}

impl<P, D> Dht22<P, D>
where
    P: InputPin + OutputPin,
    D: DelayNs,
{
    pub fn new(pin: P, delay: D) -> Self {
        Self { pin, delay }
    }

    /// Release the bus (pull-up takes the line high) and consume the driver,
    /// handing the pin + delay back.
    pub fn release(self) -> (P, D) {
        (self.pin, self.delay)
    }

    /// Trigger a conversion and read one sample.
    ///
    /// DHT22 must not be polled faster than ~0.5 Hz; enforce the >=2s spacing
    /// at the call site (the firmware's monotonic task does this).
    pub fn read(&mut self) -> Result<Reading, Error> {
        // --- Start signal: hold low ~1.1ms, then release. ---
        self.pin.set_low().map_err(|_| Error::Pin)?;
        self.delay.delay_us(1_100);
        self.pin.set_high().map_err(|_| Error::Pin)?;
        self.delay.delay_us(30);

        // --- Sensor response: ~80us low, ~80us high. ---
        self.wait_for(false, 90)?;
        self.wait_for(true, 90)?;
        self.wait_for(false, 90)?;

        // --- 40 data bits. ---
        let mut data = [0u8; 5];
        for i in 0..40 {
            // Each bit: ~50us low, then a high pulse whose length encodes
            // the value (~26-28us = 0, ~70us = 1).
            self.wait_for(true, 60)?;
            // Sample after 40us: still high => 1, already low => 0.
            self.delay.delay_us(40);
            let bit = self.pin.is_high().map_err(|_| Error::Pin)?;
            if bit {
                data[i / 8] |= 1 << (7 - (i % 8));
                // Let the long-1 pulse finish before the next bit's low.
                self.wait_for(false, 60)?;
            }
        }

        // --- Checksum. ---
        let sum = data[0]
            .wrapping_add(data[1])
            .wrapping_add(data[2])
            .wrapping_add(data[3]);
        if sum != data[4] {
            return Err(Error::Checksum);
        }

        let humidity_pct = (((data[0] as u16) << 8) | data[1] as u16) as f32 / 10.0;
        let raw_t = (((data[2] & 0x7F) as u16) << 8) | data[3] as u16;
        let mut temperature_c = raw_t as f32 / 10.0;
        if data[2] & 0x80 != 0 {
            temperature_c = -temperature_c;
        }

        Ok(Reading {
            temperature_c,
            humidity_pct,
        })
    }

    /// Busy-poll until the line reaches `level`, giving up after `timeout_us`.
    fn wait_for(&mut self, level: bool, timeout_us: u32) -> Result<(), Error> {
        let mut waited = 0;
        loop {
            let now = self.pin.is_high().map_err(|_| Error::Pin)?;
            if now == level {
                return Ok(());
            }
            if waited >= timeout_us {
                return Err(Error::Timeout);
            }
            self.delay.delay_us(1);
            waited += 1;
        }
    }
}
