use core::sync::atomic::{AtomicBool, AtomicI32, AtomicU32, AtomicU16, AtomicU8, Ordering};

use bit_field::BitField;

use crate::gpio::{AfType, Pull};
use crate::pac::gpio::Gpio;
use crate::pac::timer::vals;
use crate::time::Hertz;
use crate::timer::low_level::{FilterValue, InputCaptureMode, InputTISelection, SlaveMode, Timer, TriggerSource};
use crate::timer::{Ch1, Ch2, Ch3, Channel, GeneralInstance4Channel, TimerPin};
use crate::Peri;


/// 3-bit hall sensor pattern (ABC).
#[derive(Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u8)]
pub enum HallPattern {
    P000 = 0,
    P001 = 1,
    P010 = 2,
    P011 = 3,
    P100 = 4,
    P101 = 5,
    P110 = 6,
    P111 = 7
}

/// Mapping from 6 hall sensor patterns to electrical angles in degrees.
///
/// Constructed from an array of 6 `(HallPattern, angle)` pairs covering
/// all valid sensor states. Patterns not listed are treated as invalid at runtime.
///
/// ```ignore
/// HallMap::new([
///     (HallPattern::P101, 0),
///     (HallPattern::P100, 60),
///     (HallPattern::P110, 120),
///     (HallPattern::P010, 180),
///     (HallPattern::P011, 240),
///     (HallPattern::P001, 300),
/// ])
/// ```
#[derive(Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct HallMap {
    entries: [(HallPattern, u16); 6],
}

impl HallMap {
    /// Create a hall map from exactly 6 (pattern, angle) pairs.
    pub const fn new(entries: [(HallPattern, u16); 6]) -> Self {
        Self { entries }
    }

    /// Returns the angle for a pattern, or `None` if the pattern was not configured.
    #[inline]
    fn get(&self, pattern: u8) -> Option<u16> {
        let mut i = 0;
        while i < 6 {
            if self.entries[i].0 as u8 == pattern {
                return Some(self.entries[i].1);
            }
            i += 1;
        }
        None
    }
}

impl Default for HallMap {
    fn default() -> Self {
        Self::new([
            (HallPattern::P101, 0),
            (HallPattern::P100, 60),
            (HallPattern::P110, 120),
            (HallPattern::P010, 180),
            (HallPattern::P011, 240),
            (HallPattern::P001, 300),
        ])
    }
}

/// Hall sensor configuration.
#[derive(Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Config {
    /// Pull resistor configuration for hall input pins.
    pub pull: Pull,
    /// Digital filter applied to hall inputs (TI1F).
    pub filter: FilterValue,
    /// Mapping from hall sensor patterns to electrical angles.
    pub hall_map: HallMap,
    /// The counting frequency
    pub tim_freq: Hertz,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            pull: Pull::None,
            filter: FilterValue::NO_FILTER,
            hall_map: HallMap::default(),
            tim_freq: Hertz(1_000_000),
        }
    }
}

/// Snapshot of hall sensor state
pub struct HallState {
    /// angle of current hall sector (0, 60, ..., 300)
    pub hall_sector: u16,
    /// angle between current sector and next sector, with sign indicating current direction of rotation
    pub hall_span: i16,
    /// previous timer count (period) between hall edges, scaled by 2^32 (i.e. 1<<32 / period)
    pub period_recip: u32,
    /// current timer count since last hall edge (i.e. ratio_q16 = (counter*period_recip) >> 16)
    pub counter: u32,
    /// `true` if the last sensor reading was an unmapped hall pattern (e.g. 000 or 111)
    pub invalid_pattern: bool,
}

/// Hall sensor driver
pub struct HallSensor<'d, T: GeneralInstance4Channel> {
    inner: Timer<'d, T>,

    /// GPIO register block shared by all three hall pins
    gpio: Gpio,
    pin_a: u8,
    pin_b: u8,
    pin_c: u8,

    hall_map: HallMap,

    // --- Atomic state written and read from different ISRs ---
    /// count of consecutive update events (overflows) since a hal edge
    overflows: AtomicU16,
    /// Most recent 3-bit hall pattern
    pattern: AtomicU8,
    /// Hall period value reciprocal (1<<32 / (overflows*1<<16 + counter) at last hall edge
    period_recip: AtomicU32,
    /// signed angular span to the next hall sector, computed at each hall edge
    hall_span: AtomicI32,
    /// `true` if the last sensor reading was an unmapped hall pattern
    invalid_pattern: AtomicBool,
}

// Safety for Sync: all mutable state is in atomics
unsafe impl<T: GeneralInstance4Channel> Sync for HallSensor<'_, T> {}

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

        let mut inner = Timer::new(tim);
        let regs = inner.regs_gp16();
        inner.set_tick_freq(config.tim_freq);

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
            hall_map: config.hall_map,
            overflows: AtomicU16::new(0),
            pattern: AtomicU8::new(0),
            period_recip: AtomicU32::new(0),
            hall_span: AtomicI32::new(0),
            invalid_pattern: AtomicBool::new(false),
        }
    }

    /// Call this from the timer peripherals interrupt handler
    pub fn on_interrupt(&self) {
        if self.inner.get_update_interrupt() {
            self.on_update_interrupt();
        } else if self.inner.get_input_interrupt(Channel::Ch1) {
            self.on_input_interrupt();
        }
    }

    /// Overflow events indicate motor stall or insufficient prescaling,
    /// this handler tracks consecutive overflows to account for them in the hall period
    fn on_update_interrupt(&self) {
        let current = self.overflows.load( Ordering::Relaxed);
        if current < u16::MAX {
            self.overflows.store(current + 1, Ordering::Relaxed);
        }
        self.inner.clear_update_interrupt();
    }

    /// Compute the hall period and store the hall state on each XOR edge event
    fn on_input_interrupt(&self) {
        let regs = self.inner.regs_gp16();
        
        // Workaround for race condition between read, update ISR, and counter clear by HW
        let mut overflows;
        let mut captured;
        let mut update_isr_active;
        loop {
            overflows = self.overflows.load(Ordering::Relaxed);
            captured = regs.ccr(0).read().0 as u16;
            update_isr_active = self.inner.get_update_interrupt();
            let tmp = self.overflows.load(Ordering::Relaxed);
            if overflows == tmp {
                break;
            }
        }
        if update_isr_active {
            // consume the interupt, the update ISR should not run and increment after a hall edge occured 
            self.inner.clear_update_interrupt();
            overflows += 1;
        }
        let period : u32 = ((overflows as u32) << 16) | (captured as u32);

        // Read hall pin states from GPIO IDR in a single bus access
        let old_pattern = self.pattern.load(Ordering::Relaxed);
        let idr = self.gpio.idr().read().0;
        let ha = idr.get_bit(self.pin_a as usize) as u8;
        let hb = idr.get_bit(self.pin_b as usize) as u8;
        let hc = idr.get_bit(self.pin_c as usize) as u8;
        let pattern = ha | (hb << 1) | (hc << 2);

        // Check if the new pattern is mapped
        let new_angle = self.hall_map.get(pattern);
        self.invalid_pattern.store(new_angle.is_none(), Ordering::Relaxed);

        // Compute signed span from previous sector to this one (shortest path around 360)
        let old_angle = self.hall_map.get(old_pattern);
        let span = if let (Some(old), Some(new)) = (old_angle, new_angle) {
            let mut delta = new as i32 - old as i32;
            if delta > 180 {
                delta -= 360;
            }
            if delta < -180 {
                delta += 360;
            }
            delta
        } else {
            0
        };
        self.hall_span.store(span, Ordering::Relaxed);

        let mut period_recip: u32 = 0;
        if period > 0 {
            period_recip = u32::MAX / period;
        }
        self.pattern.store(pattern, Ordering::Relaxed);
        self.period_recip.store(period_recip, Ordering::Relaxed);
        self.inner.clear_input_interrupt(Channel::Ch1);
        self.overflows.store(0, Ordering::Relaxed);
    }

    /// Takes a snapshot of the current hall state
    #[inline]
    pub fn read_state(&self) -> HallState {
        let mut overflows;
        let mut counter;
        let mut update_isr_active;
        // Workaround for race condition between read, update ISR, and counter clear by HW
        loop {
            overflows = self.overflows.load(Ordering::Relaxed);
            counter = self.inner.regs_gp16().cnt().read().0 as u16;
            update_isr_active = self.inner.get_update_interrupt();
            let tmp = self.overflows.load(Ordering::Relaxed);
            if overflows == tmp {
                break;
            }
        }
        // Do only local increment here, and let update ISR do its own increment by not clearing the ISR flag
        if update_isr_active {
            overflows += 1;
        }
        let count : u32 = (overflows as u32) * (u16::MAX as u32) + (counter as u32);

        let pattern = self.pattern.load(Ordering::Relaxed);
        let invalid = self.invalid_pattern.load(Ordering::Relaxed);
        let cur_angle = self.hall_map.get(pattern).unwrap_or(0);
        let hall_span = self.hall_span.load(Ordering::Relaxed) as i16;

        HallState {
            hall_sector: pattern as u16,
            hall_span,
            period_recip: self.period_recip.load(Ordering::Relaxed),
            counter: count,
            invalid_pattern: invalid,
        }
    }
}
