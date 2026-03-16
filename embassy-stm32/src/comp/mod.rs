//! Comparator (COMP)
#![macro_use]
use embassy_hal_internal::PeripheralType;
use crate::Peri;

mod inmsel;
pub use inmsel::*;
mod inpsel;
pub use inpsel::*;
mod blanksel;
pub use blanksel::*;

pub(crate) trait SealedInmSelect<T: Instance>: Into<u8> {}
pub(crate) trait SealedInpSelect<T: Instance>: Into<bool> {}
pub(crate) trait SealedBlankSelect<T: Instance>: Into<u8> {}

#[allow(private_bounds)]
pub trait InmSelect<T: Instance>: SealedInmSelect<T> {}
#[allow(private_bounds)]
pub trait InpSelect<T: Instance>: SealedInpSelect<T> {}
#[allow(private_bounds)]
pub trait BlankSelect<T: Instance>: SealedBlankSelect<T> {}

macro_rules! impl_comp_bindings {
    ($($inst:ident: $inm:ty, $inp:ty, $blank:ty;)*) => {$(
        impl SealedInmSelect<crate::peripherals::$inst> for $inm {}
        impl InmSelect<crate::peripherals::$inst> for $inm {}
        impl SealedInpSelect<crate::peripherals::$inst> for $inp {}
        impl InpSelect<crate::peripherals::$inst> for $inp {}
        impl SealedBlankSelect<crate::peripherals::$inst> for $blank {}
        impl BlankSelect<crate::peripherals::$inst> for $blank {}
    )*};
}

#[cfg(stm32g4)]
mod _g4_bindings {
    use super::*;
    impl_comp_bindings! {
        COMP1: Comp1InmSel, Comp1InpSel, Comp1BlankSel;
        COMP2: Comp2InmSel, Comp2InpSel, Comp2BlankSel;
        COMP3: Comp3InmSel, Comp3InpSel, Comp3BlankSel;
        COMP4: Comp4InmSel, Comp4InpSel, Comp4BlankSel;
        COMP5: Comp5InmSel, Comp5InpSel, Comp5BlankSel;
        COMP6: Comp6InmSel, Comp6InpSel, Comp6BlankSel;
        COMP7: Comp7InmSel, Comp7InpSel, Comp7BlankSel;
    }
}

pub(crate) trait SealedInstance {
    fn regs() -> crate::pac::comp::Comp;
    fn number() -> usize;
}

#[allow(private_bounds)]
pub trait Instance: SealedInstance + PeripheralType + 'static {}

macro_rules! comp_number {
    (COMP1) => { 1 };
    (COMP2) => { 2 };
    (COMP3) => { 3 };
    (COMP4) => { 4 };
    (COMP5) => { 5 };
    (COMP6) => { 6 };
    (COMP7) => { 7 };
}

foreach_peripheral! {
    (comp, $inst:ident) => {
        impl SealedInstance for crate::peripherals::$inst {
            fn regs() -> crate::pac::comp::Comp {
                crate::pac::$inst
            }

            fn number() -> usize {
                comp_number!($inst)
            }
        }

        impl Instance for crate::peripherals::$inst {}
    };
}

pub struct Comp<'a, T: Instance> {
    _inner: Peri<'a, T>
}

impl<'a, T: Instance> Comp<'a, T> {
    /// Returns the comparator number (e.g. 4 for COMP4).
    pub fn number(&self) -> usize {
        T::number()
    }

    /// Creates a new comparator instance
    pub fn new(comp: Peri<'a, T>, input: impl InpSelect<T>, complementary_input: impl InmSelect<T>, blanking: impl BlankSelect<T>) -> Self {
        let regs = T::regs();
        regs.csr().modify(|w| {
            w.set_inpsel(input.into());
            w.set_inmsel(complementary_input.into());
            w.set_blanksel(blanking.into());
            w.set_en(true);
            w.set_lock(true);
        });
        Self {
            _inner: comp
        }
    }
}