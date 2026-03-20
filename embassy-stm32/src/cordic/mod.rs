//! Coordinate Rotation Digital Computer (CORDIC)
//!
//! Typestate driver that encodes function and data width at the type level.
//!
//! # Usage
//!
//! ```rust,ignore
//! let mut cordic = Cordic::new(p.CORDIC);
//!
//! // Configure for sine/cosine, Q1.31 — always returns both results
//! let mut sin = cordic.configure::<Sin, Q31>(Precision::Iters12, NoScale);
//! let started = sin.start1(angle_q31);
//! let (sin_val, cos_val) = started.result2();
//!
//! // Reconfigure for sqrt
//! let mut sqrt = cordic.configure::<Sqrt, Q31>(Precision::Iters24, SqrtScale::N0);
//! let started = sqrt.start1(value);
//! let result = started.result();
//! ```

use core::marker::PhantomData;
use embassy_hal_internal::{Peri, PeripheralType};
use crate::pac::cordic::vals;
use crate::{peripherals, rcc};

mod enums;
mod errors;
mod functions;
pub mod utils;

pub use enums::*;
pub use errors::*;
pub use functions::*;

/// Q1.31 fixed-point format (32-bit arguments and results)
pub struct Q31;
/// Q1.15 fixed-point format (16-bit arguments and results)
pub struct Q15;

mod width_sealed {
    pub trait Sealed {}
}

/// Data width trait — controls argument/result types and hardware access patterns.
#[allow(private_bounds)]
pub trait DataWidth: width_sealed::Sealed {
    /// Argument type exposed to user
    type Arg: Copy + Default;
    /// Result type exposed to user
    type Res: Copy;

    #[doc(hidden)]
    const ARGSIZE: vals::Size;
    #[doc(hidden)]
    const RESSIZE: vals::Size;
    /// +1 in the respective fixed-point format, used for ARG2 initialization
    #[doc(hidden)]
    const ARG2_PLUS_ONE: Self::Arg;
    /// NRES value for two-result reads: NUM1 for Q15 (packed in one read), NUM2 for Q31.
    #[doc(hidden)]
    const NRES_TWO: vals::Num;
    /// NARGS value for two-argument writes: NUM1 for Q15 (packed in one write), NUM2 for Q31.
    #[doc(hidden)]
    const NARGS_TWO: vals::Num;

    #[doc(hidden)]
    fn write_one_arg(regs: crate::pac::cordic::Cordic, arg: Self::Arg);
    #[doc(hidden)]
    fn write_two_args(regs: crate::pac::cordic::Cordic, arg1: Self::Arg, arg2: Self::Arg);
    #[doc(hidden)]
    fn read_one_res(regs: crate::pac::cordic::Cordic) -> Self::Res;
    #[doc(hidden)]
    fn read_two_res(regs: crate::pac::cordic::Cordic) -> (Self::Res, Self::Res);
}

impl width_sealed::Sealed for Q31 {}
impl DataWidth for Q31 {
    type Arg = u32;
    type Res = u32;

    const ARGSIZE: vals::Size = vals::Size::BITS32;
    const RESSIZE: vals::Size = vals::Size::BITS32;
    const ARG2_PLUS_ONE: u32 = 0x7FFF_FFFF;
    const NRES_TWO: vals::Num = vals::Num::NUM2;
    const NARGS_TWO: vals::Num = vals::Num::NUM2;

    #[inline(always)]
    fn write_one_arg(regs: crate::pac::cordic::Cordic, arg: u32) {
        regs.wdata().write_value(arg);
    }
    #[inline(always)]
    fn write_two_args(regs: crate::pac::cordic::Cordic, arg1: u32, arg2: u32) {
        regs.wdata().write_value(arg1);
        regs.wdata().write_value(arg2);
    }
    #[inline(always)]
    fn read_one_res(regs: crate::pac::cordic::Cordic) -> u32 {
        regs.rdata().read()
    }
    #[inline(always)]
    fn read_two_res(regs: crate::pac::cordic::Cordic) -> (u32, u32) {
        let r1 = regs.rdata().read();
        let r2 = regs.rdata().read();
        (r1, r2)
    }
}

impl width_sealed::Sealed for Q15 {}
impl DataWidth for Q15 {
    type Arg = u16;
    type Res = u16;

    const ARGSIZE: vals::Size = vals::Size::BITS16;
    const RESSIZE: vals::Size = vals::Size::BITS16;
    const ARG2_PLUS_ONE: u16 = 0x7FFF;
    const NRES_TWO: vals::Num = vals::Num::NUM1;
    const NARGS_TWO: vals::Num = vals::Num::NUM1;

    #[inline(always)]
    fn write_one_arg(regs: crate::pac::cordic::Cordic, arg: u16) {
        regs.wdata().write_value(arg as u32);
    }
    #[inline(always)]
    fn write_two_args(regs: crate::pac::cordic::Cordic, arg1: u16, arg2: u16) {
        // Q15: both args packed in one write — arg1 low, arg2 high
        regs.wdata().write_value((arg1 as u32) | ((arg2 as u32) << 16));
    }
    #[inline(always)]
    fn read_one_res(regs: crate::pac::cordic::Cordic) -> u16 {
        regs.rdata().read() as u16
    }
    #[inline(always)]
    fn read_two_res(regs: crate::pac::cordic::Cordic) -> (u16, u16) {
        let raw = regs.rdata().read();
        (raw as u16, (raw >> 16) as u16)
    }
}

trait SealedInstance {
    fn regs() -> crate::pac::cordic::Cordic;
}

/// CORDIC peripheral instance
#[allow(private_bounds)]
pub trait Instance: SealedInstance + PeripheralType + rcc::RccPeripheral {}

pub struct Cordic<'d, T: Instance> {
    peri: Peri<'d, T>,
}

/// Configured CORDIC handle. Borrows the driver for a specific function and data width.
/// Reusable for multiple start/result cycles with the same configuration.
pub struct Configured<'a, 'd, T: Instance, F: FunctionType, W: DataWidth> {
    cordic: &'a mut Cordic<'d, T>,
    arg2_loaded: bool,
    _phantom: PhantomData<(F, W)>,
}

/// Started CORDIC computation. Borrows through `Configured`, preventing new starts
/// until the result is read.
#[must_use = "CORDIC result must be read before starting a new computation"]
pub struct Started<'a, 'd, T: Instance, F: FunctionType, W: DataWidth> {
    cordic: &'a mut Cordic<'d, T>,
    _phantom: PhantomData<(F, W)>,
}

impl<'d, T: Instance> Cordic<'d, T> {
    /// Create a new CORDIC driver, enabling the peripheral clock.
    pub fn new(peri: Peri<'d, T>) -> Self {
        rcc::enable_and_reset::<T>();
        Self { peri }
    }

    /// Configure the CORDIC for a specific function and data width.
    ///
    /// Returns a [`Configured`] handle that borrows this driver.
    pub fn configure<F: FunctionType, W: DataWidth>(
        &mut self,
        precision: Precision,
        scale: F::Scale,
    ) -> Configured<'_, 'd, T, F, W> {
        let regs = T::regs();
        regs.csr().modify(|v| {
            v.set_func(F::FUNC);
            v.set_precision(vals::Precision::from_bits(precision as u8));
            v.set_scale(vals::Scale::from_bits(scale.raw()));
            v.set_argsize(W::ARGSIZE);
            v.set_ressize(W::RESSIZE);
        });

        Configured {
            cordic: self,
            arg2_loaded: false,
            _phantom: PhantomData,
        }
    }
}

impl<'d, T: Instance> Drop for Cordic<'d, T> {
    fn drop(&mut self) {
        rcc::disable::<T>();
    }
}

impl<'a, 'd, T: Instance, F: FunctionType, W: DataWidth> Configured<'a, 'd, T, F, W> {
    /// Start a computation. For two-arg functions, ARG2 defaults to +1 (unit modulus) on
    /// the first call and is retained by the hardware on subsequent calls. 
    pub fn start1(&mut self, arg: W::Arg) -> Started<'_, 'd, T, F, W> {
        let nres = if F::TWO_RES { W::NRES_TWO } else { vals::Num::NUM1 };
        let regs = T::regs();
        if F::TWO_ARGS && !self.arg2_loaded {
            regs.csr().modify(|v| { v.set_nargs(W::NARGS_TWO); v.set_nres(nres); });
            W::write_two_args(regs, arg, W::ARG2_PLUS_ONE);
            self.arg2_loaded = true;
        } else {
            regs.csr().modify(|v| { v.set_nargs(vals::Num::NUM1); v.set_nres(nres); });
            W::write_one_arg(regs, arg);
        }
        Started { cordic: &mut *self.cordic, _phantom: PhantomData }
    }
}

impl<'a, 'd, T: Instance, F: FnTwoArgs, W: DataWidth> Configured<'a, 'd, T, F, W> {
    /// Start a computation with two explicit arguments.
    pub fn start2(&mut self, arg1: W::Arg, arg2: W::Arg) -> Started<'_, 'd, T, F, W> {
        let nres = if F::TWO_RES { W::NRES_TWO } else { vals::Num::NUM1 };
        let regs = T::regs();
        regs.csr().modify(|v| { v.set_nargs(W::NARGS_TWO); v.set_nres(nres); });
        W::write_two_args(regs, arg1, arg2);
        self.arg2_loaded = true;
        Started { cordic: &mut *self.cordic, _phantom: PhantomData }
    }
}

impl<'a, 'd, T: Instance, F: FnOneRes, W: DataWidth> Started<'a, 'd, T, F, W> {
    /// Read the primary result. Stalls the AHB bus until the computation is ready.
    pub fn result(self) -> W::Res {
        W::read_one_res(T::regs())
    }
}

impl<'a, 'd, T: Instance, F: FnTwoRes, W: DataWidth> Started<'a, 'd, T, F, W> {
    /// Read both results. Stalls the AHB bus until the computation is ready.
    pub fn result2(self) -> (W::Res, W::Res) {
        W::read_two_res(T::regs())
    }
}

foreach_interrupt!(
    ($inst:ident, cordic, $block:ident, GLOBAL, $irq:ident) => {
        impl Instance for peripherals::$inst {}

        impl SealedInstance for peripherals::$inst {
            fn regs() -> crate::pac::cordic::Cordic {
                crate::pac::$inst
            }
        }
    };
);

dma_trait!(WriteDma, Instance);
dma_trait!(ReadDma, Instance);
