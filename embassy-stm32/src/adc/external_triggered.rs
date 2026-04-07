use stm32_metapac::adc::regs::{Jsqr, Sqr1, Sqr2, Sqr3, Sqr4};
use super::{SampleTime};
use crate::adc::Adc;
use crate::pac::adc::vals;
#[allow(unused_imports)]
use crate::adc::Instance;

pub enum EocInterruptEnabled {
    /// End of regular conversion interrupt disabled
    DISABLED,
    /// End of regular conversion interrupt enabled
    ENABLED
}

pub enum JeosInterruptEnabled {
    /// End of injected conversion sequence interrupt disabled
    DISABLED,
    /// End of injected conversion sequence interrupt enabled
    ENABLED
}

pub enum StartMode {
    /// Start the queued instance with a nonempty queue (user called with_injected_sequence)
    NORMAL,
    /// Start the queued instance with an empty queue
    EMPTY
}

pub trait HasInjectedTrigger {
    type Trigger;
    fn jextsel(trigger: Self::Trigger) -> u8;
}

pub trait HasRegularTrigger {
    type Trigger;
    fn extsel(trigger: Self::Trigger) -> u8;
}

#[repr(u8)]
pub enum Adc12RegularTrigger {
    Tim1Oc1       = 0b00000,
    Tim1Oc2       = 0b00001,
    Tim1Oc3       = 0b00010,
    Tim2Oc2       = 0b00011,
    Tim3Trgo      = 0b00100,
    Tim4Oc4       = 0b00101,
    Line11        = 0b00110,
    Tim8Trgo      = 0b00111,
    Tim8Trgo2     = 0b01000,
    Tim1Trgo      = 0b01001,
    Tim1Trgo2     = 0b01010,
    Tim2Trgo      = 0b01011,
    Tim4Trgo      = 0b01100,
    Tim6Trgo      = 0b01101,
    Tim15Trgo     = 0b01110,
    Tim3Oc4       = 0b01111,
    Tim20Trgo     = 0b10000,
    Tim20Trgo2    = 0b10001,
    Tim20Oc1      = 0b10010,
    Tim20Oc2      = 0b10011,
    Tim20Oc3      = 0b10100,
    HrtimAdcTrg1  = 0b10101,
    HrtimAdcTrg3  = 0b10110,
    HrtimAdcTrg5  = 0b10111,
    HrtimAdcTrg6  = 0b11000,
    HrtimAdcTrg7  = 0b11001,
    HrtimAdcTrg8  = 0b11010,
    HrtimAdcTrg9  = 0b11011,
    HrtimAdcTrg10 = 0b11100,
    LptimOut      = 0b11101,
    Tim7Trgo      = 0b11110,
}

#[repr(u8)]
pub enum Adc345RegularTrigger {
    Tim3Oc1       = 0b00000,
    Tim2Oc3       = 0b00001,
    Tim1Oc3       = 0b00010,
    Tim8Oc1       = 0b00011,
    Tim3Trgo      = 0b00100,
    Line2         = 0b00101,
    Tim4Oc1       = 0b00110,
    Tim8Trgo      = 0b00111,
    Tim8Trgo2     = 0b01000,
    Tim1Trgo      = 0b01001,
    Tim1Trgo2     = 0b01010,
    Tim2Trgo      = 0b01011,
    Tim4Trgo      = 0b01100,
    Tim6Trgo      = 0b01101,
    Tim15Trgo     = 0b01110,
    Tim2Oc1       = 0b01111,
    Tim20Trgo     = 0b10000,
    Tim20Trgo2    = 0b10001,
    Tim20Oc1      = 0b10010,
    HrtimAdcTrg2  = 0b10011,
    HrtimAdcTrg4  = 0b10100,
    HrtimAdcTrg1  = 0b10101,
    HrtimAdcTrg3  = 0b10110,
    HrtimAdcTrg5  = 0b10111,
    HrtimAdcTrg6  = 0b11000,
    HrtimAdcTrg7  = 0b11001,
    HrtimAdcTrg8  = 0b11010,
    HrtimAdcTrg9  = 0b11011,
    HrtimAdcTrg10 = 0b11100,
    LptimOut      = 0b11101,
    Tim7Trgo      = 0b11110,
}

#[repr(u8)]
pub enum Adc12InjectedTrigger {
    Tim1Trgo      = 0b00000,
    Tim1Cc4       = 0b00001,
    Tim2Trgo      = 0b00010,
    Tim2Cc1       = 0b00011,
    Tim3Cc4       = 0b00100,
    Tim4Trgo      = 0b00101,
    Line15        = 0b00110,
    Tim8Cc4       = 0b00111,
    Tim1Trgo2     = 0b01000,
    Tim8Trgo      = 0b01001,
    Tim8Trgo2     = 0b01010,
    Tim3Cc3       = 0b01011,
    Tim3Trgo      = 0b01100,
    Tim3Cc1       = 0b01101,
    Tim6Trgo      = 0b01110,
    Tim15Trgo     = 0b01111,
    Tim20Trgo     = 0b10000,
    Tim20Trgo2    = 0b10001,
    Tim20Cc4      = 0b10010,
    HrtimAdcTrg2  = 0b10011,
    HrtimAdcTrg4  = 0b10100,
    HrtimAdcTrg5  = 0b10101,
    HrtimAdcTrg6  = 0b10110,
    HrtimAdcTrg7  = 0b10111,
    HrtimAdcTrg8  = 0b11000,
    HrtimAdcTrg9  = 0b11001,
    HrtimAdcTrg10 = 0b11010,
    Tim16Cc1      = 0b11011,
    LptimOut      = 0b11101,
    Tim7Trgo      = 0b11110,
}

#[repr(u8)]
pub enum Adc345InjectedTrigger {
    Tim1Trgo      = 0b00000,
    Tim1Cc4       = 0b00001,
    Tim2Trgo      = 0b00010,
    Tim8Cc2       = 0b00011,
    Tim4Cc3       = 0b00100,
    Tim4Trgo      = 0b00101,
    Tim4Cc4       = 0b00110,
    Tim8Cc4       = 0b00111,
    Tim1Trgo2     = 0b01000,
    Tim8Trgo      = 0b01001,
    Tim8Trgo2     = 0b01010,
    Tim1Cc3       = 0b01011,
    Tim3Trgo      = 0b01100,
    Line3         = 0b01101,
    Tim6Trgo      = 0b01110,
    Tim15Trgo     = 0b01111,
    Tim20Trgo     = 0b10000,
    Tim20Trgo2    = 0b10001,
    Tim20Cc2      = 0b10010,
    HrtimAdcTrg2  = 0b10011,
    HrtimAdcTrg4  = 0b10100,
    HrtimAdcTrg5  = 0b10101,
    HrtimAdcTrg6  = 0b10110,
    HrtimAdcTrg7  = 0b10111,
    HrtimAdcTrg8  = 0b11000,
    HrtimAdcTrg9  = 0b11001,
    HrtimAdcTrg10 = 0b11010,
    HrtimAdcTrg1  = 0b11011,
    HrtimAdcTrg3  = 0b11100,
    LptimOut      = 0b11101,
    Tim7Trgo      = 0b11110,
}

macro_rules! impl_injected_trigger {
    ($trigger_enum:ty, $($adc:ty),+) => {
        $(
            impl HasInjectedTrigger for $adc {
                type Trigger = $trigger_enum;
                
                fn jextsel(trigger: Self::Trigger) -> u8 {
                    trigger as u8
                }
            }
        )+
    }
}
impl_injected_trigger!(Adc12InjectedTrigger, crate::peripherals::ADC1, crate::peripherals::ADC2);
impl_injected_trigger!(Adc345InjectedTrigger, crate::peripherals::ADC3, crate::peripherals::ADC4, crate::peripherals::ADC5);

macro_rules! impl_regular_trigger {
    ($trigger_enum:ty, $($adc:ty),+) => {
        $(
            impl HasRegularTrigger for $adc {
                type Trigger = $trigger_enum;

                fn extsel(trigger: Self::Trigger) -> u8 {
                    trigger as u8
                }
            }
        )+
    }
}
impl_regular_trigger!(Adc12RegularTrigger, crate::peripherals::ADC1, crate::peripherals::ADC2);
impl_regular_trigger!(Adc345RegularTrigger, crate::peripherals::ADC3, crate::peripherals::ADC4, crate::peripherals::ADC5);

pub struct NotRunning;
pub struct Running;
pub struct NotQueued;
pub struct Queued;

pub struct ExternalTriggeredADC<'a, T: Instance<Regs = crate::pac::adc::Adc>, RUNNING, QUEUED> {
    running: RUNNING,
    queued: QUEUED,
    inner: Adc<'a, T>,
    regular_configured: bool,
    injected_configured: bool,
}

impl<'a, T: Instance<Regs = crate::pac::adc::Adc>, RUNNING, QUEUED> ExternalTriggeredADC<'a, T, RUNNING, QUEUED> {
    pub fn new(adc: Adc<'a, T>) -> ExternalTriggeredADC<'a, T, NotRunning, NotQueued> {
        let regs = T::regs();
        regs.cfgr().modify(|w| w.set_jauto(false));
        ExternalTriggeredADC { 
            inner: adc, running: NotRunning, queued: NotQueued,
            regular_configured: false, injected_configured: false, 
        }
    }

    pub fn new_with_queue(adc: Adc<'a, T>) -> ExternalTriggeredADC<'a, T, NotRunning, Queued> {
        let regs = T::regs();
        regs.cfgr().modify(|w| {
            w.set_jauto(false);
            w.set_jqm(vals::Jqm::MODE1);
            w.set_jqdis(false);
        });
        ExternalTriggeredADC { 
            inner: adc, running: NotRunning, queued: Queued,
            regular_configured: false, injected_configured: false
        }
    }

    fn write_injected(&self, 
        channels: &[u8],
        trigger_source: u8,
        trigger_type: vals::Exten
    ) {
        if channels.len() > 4 {
            panic!("The maximum length of injected context is 4")
        }
        let mut register = Jsqr::default();
        for i in 0..channels.len() {
            register.set_jsq(i, channels[i]);
        }
        register.set_jextsel(trigger_source);
        register.set_jexten(trigger_type);
        register.set_jl((channels.len() - 1) as u8);
        T::regs().jsqr().write_value(register);
    }

    /// Returns whether the end of regular conversion status bit is set
    pub fn check_eoc(&self) -> bool {
        return T::regs().isr().read().eoc();
    }

    /// Returns whether the end of injected conversion sequence status bit is set
    pub fn check_jeos(&self) -> bool {
        return T::regs().isr().read().jeos();
    }

    fn internal_start(self, 
        eoc_enabled: EocInterruptEnabled, jeos_enabled: JeosInterruptEnabled, empty_queue: bool
    ) -> ExternalTriggeredADC<'a, T, Running, QUEUED> {
        if self.regular_configured {
            if matches!(eoc_enabled, EocInterruptEnabled::ENABLED) {
                T::regs().ier().modify(|w| w.set_eocie(true));
            }
            T::regs().cr().modify(|w| w.set_adstart(true));
        }
        if self.injected_configured {
            if matches!(jeos_enabled, JeosInterruptEnabled::ENABLED) {
                T::regs().ier().modify(|w| w.set_jeosie(true));
            }
            T::regs().cr().modify(|w| w.set_jadstart(true));
            if empty_queue { // there is a dummy conversion in queue to avoid jadstart causing SW trigger
                T::regs().cr().modify(|w| w.set_jadstp(vals::Adstp::STOP));
                while T::regs().cr().read().jadstart() {}
                // Now we jadstart = true, without anything pending in the queue:
                T::regs().cr().modify(|w| w.set_jadstart(true));
            }
        }
        ExternalTriggeredADC { 
            inner: self.inner, running: Running, queued: self.queued,
            regular_configured: self.regular_configured, injected_configured: self.injected_configured, 
        }
    }
}

impl<'a, T: Instance<Regs = crate::pac::adc::Adc>, QUEUED> ExternalTriggeredADC<'a, T, NotRunning, QUEUED> {
    /// Configures the instance with the given sample times per channel
    pub fn using_sampletimes(self, sampletimes: &[(u8, SampleTime)]) -> Self {
        let mut smpr = T::regs().smpr().read();
        let mut smpr2 = T::regs().smpr2().read();
        for (ch, sampletime) in sampletimes {
            if *ch < 10 {
                smpr.set_smp(*ch as usize, *sampletime)
            } else {
                smpr2.set_smp((*ch-10) as usize, *sampletime)
            }
        }
        T::regs().smpr().write_value(smpr);
        T::regs().smpr2().write_value(smpr2);
        self
    }

    /// Configures a regular conversion sequence with the given trigger
    pub fn with_sequence(mut self,  
        channels: &[u8],
        trigger_source: T::Trigger,
        trigger_type: vals::Exten
    ) -> Self where T: HasRegularTrigger {
        if channels.len() > 4 {
            panic!("The maximum length of regular sequence is currently 4")
        }
        let mut register = Sqr1::default();
        for i in 0..channels.len() {
            register.set_sq(i, channels[i]);
        }
        register.set_l((channels.len() - 1) as u8);
        T::regs().sqr1().write_value(register);

        T::regs().cfgr().modify(|w| {
            w.set_extsel(T::extsel(trigger_source));
            w.set_exten(trigger_type);
        });
        self.regular_configured = true;
        self
    }

    /// Configures an injected conversion sequence with the given trigger
    pub fn with_injected_sequence(mut self,
        channels: &[u8],
        trigger_source: T::Trigger,
        trigger_type: vals::Exten
    ) -> Self where T: HasInjectedTrigger {
        self.write_injected(channels, T::jextsel(trigger_source), trigger_type);
        self.injected_configured = true;
        self
    }

    /// Configures the instance to use the specified result offset values for each channel 
    pub fn using_offsets(self,
        offsets: &[(u8, i16)],
    ) -> Self {
        if offsets.len() > 4 {
            panic!("The maximum number of offsets is 4")
        }
        for i in 0..offsets.len() {
            T::regs().ofr(i).modify(|w| {
                w.set_offset1_ch(offsets[i].0);
                w.set_offsetpos(offsets[i].1 > 0);
                w.set_offset(offsets[i].1.abs() as u16);
                w.set_offset_en(true);
            });
        }
        self
    }
}

impl<'a, T: Instance<Regs = crate::pac::adc::Adc>> ExternalTriggeredADC<'a, T, NotRunning, NotQueued> {
    pub fn start(self, 
        eoc_enabled: EocInterruptEnabled, jeos_enabled: JeosInterruptEnabled
    ) -> ExternalTriggeredADC<'a, T, Running, NotQueued> {
        self.internal_start(eoc_enabled, jeos_enabled, false)
    }
}

impl<'a, T: Instance<Regs = crate::pac::adc::Adc>> ExternalTriggeredADC<'a, T, NotRunning, Queued> {
    pub fn start(mut self, 
        eoc_enabled: EocInterruptEnabled, jeos_enabled: JeosInterruptEnabled, mode: StartMode
    ) -> ExternalTriggeredADC<'a, T, Running, Queued> {
        if matches!(mode, StartMode::EMPTY) {
            self.write_injected(&[0], 1, vals::Exten::DISABLED);
            self.injected_configured = true;
        }
        self.internal_start(eoc_enabled, jeos_enabled, true)
    }
}

impl<'a, T: Instance<Regs = crate::pac::adc::Adc>, QUEUED> ExternalTriggeredADC<'a, T, Running, QUEUED> {
    pub fn read(&self) -> u16 {
        while !T::regs().isr().read().eoc() {}
        T::regs().dr().read().rdata() as u16
    }
    
    pub fn read_injected<const N: usize>(&self) -> [i16; N] {
        while !T::regs().isr().read().jeos() {}
        T::regs().isr().modify(|w| w.set_jeos(true));

        let mut buf = [0i16; N];
        for i in 0..N {
            buf[i] = T::regs().jdr(i).read().jdata() as i16;
        }
        buf
    }    

    pub fn stop(self) -> ExternalTriggeredADC<'a, T, NotRunning, QUEUED> {
        if self.regular_configured {
            T::regs().cr().modify(|w| w.set_adstp(vals::Adstp::STOP));
            while T::regs().cr().read().adstart() {}
        }
        if self.injected_configured {
            T::regs().cr().modify(|w| w.set_jadstp(vals::Adstp::STOP));
            while T::regs().cr().read().jadstart() {}
        }

        ExternalTriggeredADC { 
            inner: self.inner, running: NotRunning, queued: self.queued,
            regular_configured: self.regular_configured, injected_configured: self.injected_configured, 
        }
    }
}

impl<'a, T: Instance<Regs = crate::pac::adc::Adc>> ExternalTriggeredADC<'a, T, Running, Queued> {
    /// Inserts a new injected conversion sequence to the queue of context, with the specicfied trigger
    pub fn insert_injected_context(&self,
        channels: &[u8],
        trigger_source: T::Trigger,
        trigger_type: vals::Exten
    ) where T: HasInjectedTrigger {
        self.write_injected(channels, T::jextsel(trigger_source), trigger_type);
    }   
    
    pub fn queue_is_empty(&self) -> bool {
        return T::regs().jsqr().read().0 == 0;
    }
}
