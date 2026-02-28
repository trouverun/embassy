//! Trigger output timer driver.
//!
//! Configures a timer to generate TRGO (and optionally TRGO2) signals at a
//! specified frequency. Useful for triggering ADC/DAC conversions and
//! synchronizing other timers.

use embassy_hal_internal::Peri;

use super::low_level::{RoundTo, Timer};
use super::*;
use crate::pac::timer::vals;
use crate::time::Hertz;

/// Master mode selection for TRGO output.
pub use vals::Mms as MasterMode;

/// Master mode selection 2 for TRGO2 output (advanced timers only).
#[cfg(not(stm32l0))]
pub use vals::Mms2;

/// Trigger output timer driver.
///
/// Configures a timer to output TRGO event at a given frequency
pub struct BasicTrgoOutput<'d, T: BasicInstance> {
    inner: Timer<'d, T>,
}

impl<'d, T: BasicInstance> BasicTrgoOutput<'d, T> {
    /// Create a new TRGO output driver.
    ///
    /// The TRGO happens periodically at the given frequency
    pub fn new(tim: Peri<'d, T>, frequency: Hertz) -> Self {
        let inner = Timer::new(tim);

        inner.set_frequency(frequency, RoundTo::Slower);
        inner.regs_basic().cr2().modify(|w| w.set_mms(MasterMode::UPDATE));
        inner.set_autoreload_preload(true);
        inner.generate_update_event();
        inner.start();

        Self { inner }
    }
}