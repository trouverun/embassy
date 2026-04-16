use bit_field::BitField;
use crate::gpio::{AfType, Pull};
use crate::pac::gpio::Gpio;
use crate::pac::timer::vals;
use crate::time::Hertz;
use crate::timer::low_level::{FilterValue, InputCaptureMode, InputTISelection, SlaveMode, Timer, TriggerSource};
use crate::timer::{Ch1, Ch2, Ch3, Channel, GeneralInstance4Channel, TimerPin};
use crate::Peri;


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
            filter: FilterValue::NO_FILTER,
            tim_freq: Hertz(1_000_000),
        }
    }
}

/// Snapshot of hall sensor state
pub struct HallState {
    /// reciprocal of previous timer count (period between hall edges), i.e. 1.0 / period
    pub hall_period_reciprocal_cycles: f32,
    /// current timer count
    pub counter: u32,
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

    /// Count of consecutive update events (overflows) since a hal edge
    num_overflows: u16,

    /// Hall period value reciprocal (1.0 / (overflows*(2^16-1) + counter) at last hall edge
    hall_period_reciprocal_cycles: f32,

    /// 3 bit hall pattern from the most recent hall edge
    pattern: u8,

    /// 3 bit hall pattern from the previous hall edge
    prev_pattern: u8,

    /// Timer frequency in Hz
    pub timer_frequency_hz: f32,
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

        let idr = gpio.idr().read().0;
        let ha = idr.get_bit(pin_a as usize) as u8;
        let hb = idr.get_bit(pin_b as usize) as u8;
        let hc = idr.get_bit(pin_c as usize) as u8;
        let initial_pattern = ha | (hb << 1) | (hc << 2);

        // Configure alternate functions
        let af_type = AfType::input(config.pull);
        set_as_af!(ch1, af_type);
        set_as_af!(ch2, af_type);
        set_as_af!(ch3, af_type);

        let mut inner = Timer::new(tim);
        let regs = inner.regs_gp16();
        inner.set_tick_freq(config.tim_freq);
        let timer_frequency_hz = config.tim_freq.0 as f32;

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
            hall_period_reciprocal_cycles: 0.0,
            pattern: initial_pattern,
            prev_pattern: initial_pattern,
            timer_frequency_hz
        }
    }

    /// Call this from the timer peripherals interrupt handler
    pub fn on_interrupt(&mut self) {
        if self.inner.get_update_interrupt() {
            self.on_update_interrupt();
        } else if self.inner.get_input_interrupt(Channel::Ch1) {
            self.on_input_interrupt();
        }
    }

    /// This handler tracks consecutive overflows to account for them in the hall period
    fn on_update_interrupt(&mut self) {
        let num_overflows = self.num_overflows;
        if num_overflows < u16::MAX {
            self.num_overflows += 1;
        }
        self.inner.clear_update_interrupt();
    }

    /// Returns the current 3 bit hall pattern (for calibration use)
    pub fn read_hall_pattern(&self) -> u8 {
         // Read hall pin states from GPIO IDR
        let idr = self.gpio.idr().read().0;
        let ha = idr.get_bit(self.pin_a as usize) as u8;
        let hb = idr.get_bit(self.pin_b as usize) as u8;
        let hc = idr.get_bit(self.pin_c as usize) as u8;
        ha | (hb << 1) | (hc << 2)
    }

    /// Compute the hall period on each XOR edge event
    fn on_input_interrupt(&mut self) {
        let regs = self.inner.regs_gp16();
        
        // Workaround for race condition between read, update ISR, and counter clear by HW
        let mut overflows;
        let mut captured;
        let mut update_isr_active;
        loop {
            overflows = self.num_overflows;
            captured = regs.ccr(0).read().0 as u16;
            update_isr_active = self.inner.get_update_interrupt();
            let tmp = self.num_overflows;
            if overflows == tmp {
                break;
            }
        }
        if update_isr_active {
            // consume the interupt (the update ISR should not run and 
            // double increment after a hall edge occured)
            self.inner.clear_update_interrupt();
            overflows += 1;
        }
        let period: u32 = ((overflows as u32) << 16) | (captured as u32);
        let mut hall_period_reciprocal_cycles = 0.0;
        if period > 0 {
            hall_period_reciprocal_cycles = 1.0 / period as f32;
        }
        self.hall_period_reciprocal_cycles = hall_period_reciprocal_cycles;

        self.prev_pattern = self.pattern;
        self.pattern = self.read_hall_pattern();
        self.inner.clear_input_interrupt(Channel::Ch1);
        self.num_overflows = 0;
    }

    /// Takes a snapshot of the current hall state
    pub fn read_state(&self) -> HallState {
        let mut overflows;
        let mut counter;
        let mut update_isr_active;
        // Workaround for race condition between read, update ISR, and counter clear by HW
        loop {
            overflows = self.num_overflows;
            counter = self.inner.regs_gp16().cnt().read().0 as u16;
            update_isr_active = self.inner.get_update_interrupt();
            let tmp = self.num_overflows;
            if overflows == tmp {
                break;
            }
        }
        // If there is a pending ISR increment locally, 
        // let update ISR do the actual increment by not clearing the ISR flag
        if update_isr_active {
            overflows += 1;
        }
        let count = ((overflows as u32) << 16) | (counter as u32);

        HallState {
            hall_period_reciprocal_cycles: self.hall_period_reciprocal_cycles,
            counter: count,
            pattern: self.pattern,
            prev_pattern: self.prev_pattern
        }
    }
}
