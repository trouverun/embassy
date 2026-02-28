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

macro_rules! impl_comp_inmsel {
    ($inst:ident, $enum:ty) => {
        impl SealedInmSelect<crate::peripherals::$inst> for $enum {}
        impl InmSelect<crate::peripherals::$inst> for $enum {}
    };
}

macro_rules! impl_comp_inpsel {
    ($inst:ident, $enum:ty) => {
        impl SealedInpSelect<crate::peripherals::$inst> for $enum {}
        impl InpSelect<crate::peripherals::$inst> for $enum {}
    };
}

macro_rules! impl_comp_blanksel {
    ($inst:ident, $enum:ty) => {
        impl SealedBlankSelect<crate::peripherals::$inst> for $enum {}
        impl BlankSelect<crate::peripherals::$inst> for $enum {}
    };
}

#[cfg(stm32g4)]
mod _g4_bindings {
    use super::*;
    impl_comp_inmsel!(COMP1, Comp1InmSel);
    impl_comp_inmsel!(COMP2, Comp2InmSel);
    impl_comp_inmsel!(COMP3, Comp3InmSel);
    impl_comp_inmsel!(COMP4, Comp4InmSel);
    impl_comp_inmsel!(COMP5, Comp5InmSel);
    impl_comp_inmsel!(COMP6, Comp6InmSel);
    impl_comp_inmsel!(COMP7, Comp7InmSel);
    impl_comp_inpsel!(COMP1, Comp1InpSel);
    impl_comp_inpsel!(COMP2, Comp2InpSel);
    impl_comp_inpsel!(COMP3, Comp3InpSel);
    impl_comp_inpsel!(COMP4, Comp4InpSel);
    impl_comp_inpsel!(COMP5, Comp5InpSel);
    impl_comp_inpsel!(COMP6, Comp6InpSel);
    impl_comp_inpsel!(COMP7, Comp7InpSel);
    impl_comp_blanksel!(COMP1, Comp1BlankSel);
    impl_comp_blanksel!(COMP2, Comp2BlankSel);
    impl_comp_blanksel!(COMP3, Comp3BlankSel);
    impl_comp_blanksel!(COMP4, Comp4BlankSel);
    impl_comp_blanksel!(COMP5, Comp5BlankSel);
    impl_comp_blanksel!(COMP6, Comp6BlankSel);
    impl_comp_blanksel!(COMP7, Comp7BlankSel);
}

pub(crate) trait SealedInstance {
    fn regs() -> crate::pac::comp::Comp;
}

#[allow(private_bounds)]
pub trait Instance: SealedInstance + PeripheralType + 'static {}

foreach_peripheral! {
    (comp, $inst:ident) => {
        impl SealedInstance for crate::peripherals::$inst {
            fn regs() -> crate::pac::comp::Comp {
                crate::pac::$inst
            }
        }

        impl Instance for crate::peripherals::$inst {}
    };
}

pub struct Comp<'a, T: Instance> {
    _inner: Peri<'a, T>
}

impl<'a, T: Instance> Comp<'a, T> {
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