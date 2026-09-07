use stm32_metapac::timer::vals::*;
use crate::comp::{Comp, Instance as CompInstance};
use crate::gpio::{AfType, OutputType, Pull, Speed};
use crate::time::Hertz;
use crate::timer::low_level::{FilterValue, OutputCompareMode, RoundTo, Timer};
use crate::timer::{AdvancedInstance4Channel, BreakInput, BreakInputPin, Ch1, Ch2, Ch3, Ch4, Channel, CountingMode, TimerComplementaryPin, TimerPin};
use crate::Peri;


pub struct NotRunning;
pub struct Running;
pub struct Paused;


pub trait ValidDeadTime {
    fn to_nanoseconds(self) -> u32;
}
pub enum PwmDeadtime {
    Nanosecods(u32),
}
impl ValidDeadTime for PwmDeadtime {
    fn to_nanoseconds(self) -> u32 {
        match self {
            PwmDeadtime::Nanosecods(time) => time,
        }
    }
}

/// PWM derived from an advanced control timer
pub struct PWM<'a, T: AdvancedInstance4Channel, RUNNING> {
    _running: RUNNING,
    inner: Timer<'a, T>
}

impl<'a, T: AdvancedInstance4Channel> PWM<'a, T, NotRunning> {
    /// Creates a new PWM from the provided advanced control timer
    pub fn new(
        tim : Peri<'a, T>, freq : Hertz, mode : CountingMode
    ) -> Self {
        let inner = Timer::new(tim);
        // Center-aligned modes count up then down, halving the effective frequency
        let effective_freq = if matches!(mode, 
            CountingMode::CenterAlignedDownInterrupts
            | CountingMode::CenterAlignedUpInterrupts
            | CountingMode::CenterAlignedBothInterrupts)
        {
            Hertz(freq.0 * 2)
        } else {
            freq
        };
        inner.set_frequency(effective_freq, RoundTo::Faster);
        inner.set_counting_mode(mode);
        inner.set_repetition_counter(1u16);
        Self {
            _running: NotRunning,
            inner
        }
    }

    fn setup_channel(&self, channel: Channel) {
        self.inner.set_output_compare_mode(channel, OutputCompareMode::PwmMode1);
        self.inner.set_output_compare_preload(channel, true);
        self.inner.set_compare_value(channel, 0u16.into());
        self.inner.set_ois(channel, false);
        self.inner.enable_channel(channel, true);
    }

    fn setup_complementary_channel(&self, channel: Channel, oisn_val: bool) {
        self.inner.set_oisn(channel, oisn_val);
        self.inner.enable_complementary_channel(channel, true);
    }

    /// Assigns the pin as PWM channel 1
    pub fn with_ch1<P>(self, pin : Peri<'a, P>) -> Self where P : TimerPin<T, Ch1> {
        pin.set_low();
        set_as_af!(pin, AfType::output(OutputType::PushPull, Speed::VeryHigh));
        self.setup_channel(Channel::Ch1);
        self
    }
    /// Assigns the pin as PWM channel 2
    pub fn with_ch2<P>(self, pin : Peri<'a, P>) -> Self where P : TimerPin<T, Ch2> {
        pin.set_low();
        set_as_af!(pin, AfType::output(OutputType::PushPull, Speed::VeryHigh));
        self.setup_channel(Channel::Ch2);
        self
    }
    /// Assigns the pin as PWM channel 3
    pub fn with_ch3<P>(self, pin : Peri<'a, P>) -> Self where P : TimerPin<T, Ch3> {
        pin.set_low();
        set_as_af!(pin, AfType::output(OutputType::PushPull, Speed::VeryHigh));
        self.setup_channel(Channel::Ch3);
        self
    }
    /// Assigns the pin as PWM channel 4
    pub fn with_ch4<P>(self, pin : Peri<'a, P>) -> Self where P : TimerPin<T, Ch4> {
        pin.set_low();
        set_as_af!(pin, AfType::output(OutputType::PushPull, Speed::VeryHigh));
        self.setup_channel(Channel::Ch4);
        self
    }
    
    /// Assigns the pin as PWM complementary channel 1
    pub fn with_ch1n<P>(self, pin : Peri<'a, P>, oisn_val: bool) -> Self where P : TimerComplementaryPin<T, Ch1> {
        pin.set_low();
        set_as_af!(pin, AfType::output(OutputType::PushPull, Speed::VeryHigh));
        self.setup_complementary_channel(Channel::Ch1, oisn_val);
        self
    }
    /// Assigns the pin as PWM complementary channel 2
    pub fn with_ch2n<P>(self, pin : Peri<'a, P>, oisn_val: bool) -> Self where P : TimerComplementaryPin<T, Ch2> {
        pin.set_low();
        set_as_af!(pin, AfType::output(OutputType::PushPull, Speed::VeryHigh));
        self.setup_complementary_channel(Channel::Ch2, oisn_val);
        self
    }
    /// Assigns the pin as PWM complementary channel 3
    pub fn with_ch3n<P>(self, pin : Peri<'a, P>, oisn_val: bool) -> Self where P : TimerComplementaryPin<T, Ch3> {
        pin.set_low();
        set_as_af!(pin, AfType::output(OutputType::PushPull, Speed::VeryHigh));
        self.setup_complementary_channel(Channel::Ch3, oisn_val);
        self
    }
    /// Assigns the pin as PWM complementary channel 4
    pub fn with_ch4n<P>(self, pin : Peri<'a, P>, oisn_val: bool) -> Self where P : TimerComplementaryPin<T, Ch4> {
        pin.set_low();
        set_as_af!(pin, AfType::output(OutputType::PushPull, Speed::VeryHigh));
        self.setup_complementary_channel(Channel::Ch4, oisn_val);
        self
    }

    /// Set ch4 to act as a trgo2 source, occuring at every counter peak
    pub fn with_peak_trgo2_from_ch4(self) -> Self {
        self.inner.set_output_compare_mode(Channel::Ch4, OutputCompareMode::PwmMode2);
        let value = self.inner.regs_advanced().arr().read().0 - 1;
        self.inner.set_compare_value(Channel::Ch4, value.try_into().unwrap());
        self.inner.enable_channel(Channel::Ch4, true);
        self.inner.set_mms2_selection(Mms2::COMPARE_OC4);
        self
    }

    /// Configure the symmetric deadtime, given in nanoseconds
    pub fn with_deadtime<V>(self, value: V) -> Self where V: ValidDeadTime {
        self.inner.set_clock_division(Ckd::DIV1);
        let ftds = self.inner.get_clock_frequency().0;
        let cycles = (value.to_nanoseconds() as u64 * ftds as u64).div_ceil(1_000_000_000);
        
        let dtg = if cycles <= 127 {
            let bits = cycles as u8;
            0b000_00000 | bits
        } else if cycles <= 254 {
            let bits = (cycles/2-64) as u8; 
            0b100_00000 | bits
        } else if cycles <= 504{
            let bits = (cycles/8-32) as u8; 
            0b110_00000 | bits
        } else if cycles <= 1008{
            let bits = (cycles/16-32) as u8;
            0b111_00000 | bits
        } else {
            panic!("Can not satisfy deadtime constraint")
        };
        self.inner.regs_advanced().bdtr().modify(|w| w.set_dtg(dtg));
        self
    }

    /// Assign a comparator as a break1 event source
    pub fn with_break1_comp<C: CompInstance>(self, comp: &Comp<'_, C>, output_polarity: Bkp, filter: FilterValue) -> Self {
        self.inner.regs_advanced().af1().modify(|w| {
            w.set_bkcmpe(comp.number(), true);
        });
        self.inner.regs_advanced().bdtr().modify(|w| {
            w.set_bke(0, true);
            w.set_bkp(0, output_polarity);
            w.set_bkf(0, filter);
            w.set_ossi(Ossi::IDLE_LEVEL);
            w.set_ossr(Ossr::IDLE_LEVEL);
            w.set_aoe(false);
        });
        self
    }

    /// Configure the pin to act as a break2 event source for the timer
    pub fn with_break2_pin<B: BreakInput, P: BreakInputPin<T, B>>(
        self, pin: Peri<'a, P>, input_polarity: Bkinp, output_polarity: Bkp, filter: FilterValue
    ) -> Self {
        pin.set_low();
        set_as_af!(pin, AfType::input(Pull::Up));
       
        self.inner.regs_advanced().af2().modify(|w| {
            w.set_bk2inp(input_polarity);
            w.set_bk2ine(true);
        });
        self.inner.regs_advanced().bdtr().modify(|w| {
            w.set_bke(1, true);
            w.set_bkp(1, output_polarity);
            w.set_bkf(1, filter);
            w.set_ossi(Ossi::IDLE_LEVEL);
            w.set_ossr(Ossr::IDLE_LEVEL);
            w.set_aoe(false);
        });
        self
    }

    /// Run the timer and lock the configuration
    pub fn start(self) -> PWM<'a, T, Running> {
        self.inner.set_autoreload_preload(true);
        self.inner.enable_outputs();
        self.inner.generate_update_event();
        self.inner.regs_advanced().dier().modify(|w| w.set_bie(true));
        self.inner.regs_advanced().bdtr().modify(|w| w.set_lock(Lock::LEVEL1));
        self.inner.start();
        PWM {
            _running: Running,
            inner: self.inner
        }
    }
}

impl<'a, T: AdvancedInstance4Channel> PWM<'a, T, Running> {
    /// Returns the timer autoreload value (max compare value)
    pub fn get_autoreload_value(&self) -> u32 {
        self.inner.get_max_compare_value().into()
    }

    /// Set the raw compare value for a channel
    pub fn set_compare_value(&self, channel: Channel, value: u16) {
        self.inner.set_compare_value(channel, value.into());
    }

    /// Checks and clears the break1 interrupt
    /// Returns: bool indicating if the interrupt was cleared
    pub fn acknowledge_break1(&self) -> bool {
        if self.inner.regs_advanced().sr().read().bif(0) {
            self.inner.regs_advanced().sr().modify(|w| w.set_bif(0, false));
            return true
        }
        false
    }

    /// Checks and clears the break2 interrupt
    /// Returns: bool indicating if the interrupt was cleared
    pub fn acknowledge_break2(&self) -> bool {
        if self.inner.regs_advanced().sr().read().bif(1) {
            self.inner.regs_advanced().sr().modify(|w| w.set_bif(1, false));
            return true
        }
        false
    }

    /// MOE = 1
    pub fn enable(&self) {
        self.inner.set_moe(true);
    }

    /// MOE = 0
    pub fn disable(&self) {
        self.inner.set_moe(false);
    }
}