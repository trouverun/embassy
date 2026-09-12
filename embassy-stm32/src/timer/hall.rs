use bit_field::BitField;
use crate::gpio::{AfType, Pull};
use crate::pac::gpio::Gpio;
use crate::pac::timer::regs::SrGp16;
use crate::pac::timer::vals;
use crate::time::Hertz;
use crate::timer::low_level::{FilterValue, InputCaptureMode, InputTISelection, SlaveMode, Timer, TriggerSource};
use crate::timer::{Ch1, Ch2, Ch3, Channel, GeneralInstance4Channel, TimerPin};
use crate::Peri;

/// Number of ticks in one full counter cycle (ARR = u16::MAX, so 0..=65535).
const COUNTER_PERIOD: u32 = u16::MAX as u32 + 1;

/// Hall sensor configuration.
#[derive(Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Config {
    /// Pull resistor configuration for hall input pins.
    pub pull: Pull,
    /// Digital filter applied to hall inputs (TI1F).
    pub filter: FilterValue,
    /// The counting frequency
    pub tim_freq: Hertz,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            pull: Pull::None,
            filter: FilterValue::FCK_INT_N2,
            tim_freq: Hertz(1_000_000),
        }
    }
}

/// Snapshot of hall sensor state
pub struct HallState {
    /// reciprocal of previous timer count (period between hall edges), i.e. 1.0 / period
    pub hall_period_reciprocal_count: f32,
    /// current timer count
    pub extended_counter: u32,
    /// 3 bit hall pattern from the most recent hall edge
    pub pattern: u8,
    /// 3 bit hall pattern from the previous hall edge
    pub prev_pattern: u8
}

/// Hall sensor driver
pub struct HallSensor<'d, T: GeneralInstance4Channel> {
    inner: Timer<'d, T>,

    /// GPIO register block shared by all three hall pins
    gpio: Gpio,
    pin_a: u8,
    pin_b: u8,
    pin_c: u8,

    /// Count of consecutive update events (overflows) since the last hall edge
    num_overflows: u16,
    /// Hall period value reciprocal, 1.0 / (overflows*COUNTER_PERIOD + captured), at last hall edge
    hall_period_reciprocal_count: f32,
    /// 3 bit hall pattern from the most recent hall edge
    pattern: u8,
    /// 3 bit hall pattern from the previous hall edge
    prev_pattern: u8,
    /// Tick frequency of the internal timer
    frequency_hz: f32
}

impl<'d, T: GeneralInstance4Channel> HallSensor<'d, T> {
    /// Create a new hall sensor driver
    pub fn new<PA, PB, PC>(
        tim: Peri<'d, T>,
        ch1: Peri<'d, PA>,
        ch2: Peri<'d, PB>,
        ch3: Peri<'d, PC>,
        config: Config,
    ) -> Self
    where
        PA: TimerPin<T, Ch1>,
        PB: TimerPin<T, Ch2>,
        PC: TimerPin<T, Ch3>,
    {
        // Extract pin positions and GPIO block before AF config consumes the pins
        let pin_a = ch1.pin();
        let pin_b = ch2.pin();
        let pin_c = ch3.pin();
        debug_assert!(
            ch1.port() == ch2.port() && ch2.port() == ch3.port(),
            "Hall sensor pins must be on the same GPIO port"
        );
        let gpio = ch1.block();

        // Configure alternate functions
        let af_type = AfType::input(config.pull);
        set_as_af!(ch1, af_type);
        set_as_af!(ch2, af_type);
        set_as_af!(ch3, af_type);

        let idr = gpio.idr().read().0;
        let ha = idr.get_bit(pin_a as usize) as u8;
        let hb = idr.get_bit(pin_b as usize) as u8;
        let hc = idr.get_bit(pin_c as usize) as u8;
        let initial_pattern = ha | (hb << 1) | (hc << 2);

        let mut inner = Timer::new(tim);
        let regs = inner.regs_gp16();
        inner.set_tick_freq(config.tim_freq);
        // Actual achieved tick frequency:
        let psc = regs.psc().read() as u32 + 1;
        let frequency_hz = (inner.get_clock_frequency().0 / psc) as f32;

        // TI1S = 1: XOR CH1/CH2/CH3 onto TI1.
        regs.cr2().modify(|w| w.set_ti1s(vals::Ti1s::XOR));

        // CH1 as input capture (normal TI1 mapping) with the configured filter
        inner.set_input_ti_selection(Channel::Ch1, InputTISelection::Normal);
        inner.set_input_capture_mode(Channel::Ch1, InputCaptureMode::BothEdges);
        inner.set_input_capture_filter(Channel::Ch1, config.filter);
        inner.enable_channel(Channel::Ch1, true);

        // Slave mode: reset counter on TI1F_ED (any hall edge)
        inner.set_trigger_source(TriggerSource::TI1F_ED);
        inner.set_slave_mode(SlaveMode::RESET_MODE);
        // Prevent update event on edge-triggered reset
        regs.cr1().modify(|w| w.set_urs(vals::Urs::COUNTER_ONLY));

        // ARR = max so the counter free-runs between edges
        regs.arr().write(|w| w.set_arr(u16::MAX));
        inner.set_autoreload_preload(false);

        // Enable interrupts
        inner.enable_input_interrupt(Channel::Ch1, true);
        inner.enable_update_interrupt(true);

        // Load shadow registers, then start
        inner.generate_update_event();
        inner.start();

        Self {
            inner,
            gpio,
            pin_a,
            pin_b,
            pin_c,
            num_overflows: 0,
            hall_period_reciprocal_count: 0.0,
            pattern: initial_pattern,
            prev_pattern: initial_pattern,
            frequency_hz
        }
    }

    /// Returns the configured tick frequency of the underlying timer
    pub fn get_tick_frequency_hz(&self) -> f32 {
        self.frequency_hz
    }

    /// Services the update and input interrupts. Must not interleave with `read_state`.
    #[inline(always)]
    pub fn on_interrupt(&mut self) {
        let regs = self.inner.regs_gp16();
        if regs.sr().read().uif() {
            self.on_update_interrupt();
        }
        if regs.sr().read().ccif(0) {
            self.on_input_interrupt();
        }
    }

    /// This handler tracks consecutive overflows to account for them in the hall period
    fn on_update_interrupt(&mut self) {
        let mut sr = SrGp16(u32::MAX);
        sr.set_uif(false);
        self.inner.regs_gp16().sr().write_value(sr);
        self.num_overflows = self.num_overflows.saturating_add(1);
    }

    /// Returns the current 3 bit hall pattern (for calibration use)
    pub fn read_hall_pattern(&self) -> u8 {
        let idr = self.gpio.idr().read().0;
        let ha = idr.get_bit(self.pin_a as usize) as u8;
        let hb = idr.get_bit(self.pin_b as usize) as u8;
        let hc = idr.get_bit(self.pin_c as usize) as u8;
        ha | (hb << 1) | (hc << 2)
    }

    /// Compute the hall period on each XOR edge event
    fn on_input_interrupt(&mut self) {
        let regs = self.inner.regs_gp16();

        // Clear before reading CCR1 so an edge arriving during this handler re-triggers the interrupt
        let mut sr = SrGp16(u32::MAX);
        sr.set_ccif(0, false);
        sr.set_ccof(0, false);
        regs.sr().write_value(sr);
        let captured = regs.ccr(0).read().0 as u16;
        let pattern = self.read_hall_pattern();

        // A pending overflow belongs to the ended period
        let mut overflows = self.num_overflows;
        if regs.sr().read().uif() {
            let mut sr = SrGp16(u32::MAX);
            sr.set_uif(false);
            regs.sr().write_value(sr);
            overflows = overflows.saturating_add(1);
        }

        let period: u32 = captured as u32 + (overflows as u32 * COUNTER_PERIOD);
        self.hall_period_reciprocal_count = if period > 0 { 1.0 / period as f32 } else { 0.0 };

        self.prev_pattern = self.pattern;
        self.pattern = pattern;
        self.num_overflows = 0;
    }

    /// Takes a snapshot of the current hall state. Must not interleave with `on_interrupt`.
    pub fn read_state(&mut self) -> HallState {
        let regs = self.inner.regs_gp16();
        let mut counter;
        let mut attempts = 0;
        loop {
            self.on_interrupt();
            counter = regs.cnt().read().0 as u16;
            let sr = regs.sr().read();
            // Edge or overflow landed after servicing: counter is inconsistent with the state, retry
            if !(sr.uif() || sr.ccif(0)) || attempts >= 2 {
                break;
            }
            attempts += 1;
        }
        let count = counter as u32 + (self.num_overflows as u32 * COUNTER_PERIOD);

        HallState {
            hall_period_reciprocal_count: self.hall_period_reciprocal_count,
            extended_counter: count,
            pattern: self.pattern,
            prev_pattern: self.prev_pattern
        }
    }
}
