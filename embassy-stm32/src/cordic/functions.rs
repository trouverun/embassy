//! CORDIC function type definitions
//!
//! Each CORDIC function is a zero-sized type that implements [`FunctionType`]
//! and the appropriate marker traits ([`FnTwoArgs`], [`FnOneRes`], [`FnTwoRes`])
//! to control which `start` and `result` methods are available at compile time.
//!
//! Scale is an associated type on each function, ensuring only valid scale values
//! can be passed at compile time.

use crate::pac::cordic::vals;

mod sealed {
    pub trait Sealed {}
}

/// Trait for scale values that can be written to the SCALE register field.
pub trait ScaleValue: Copy {
    /// Raw register value for the SCALE field
    fn raw(self) -> u8;
}

/// Scale not applicable — fixed at 0. Used by Sin, Cos, Phase, Modulus.
#[derive(Clone, Copy)]
pub struct NoScale;
impl ScaleValue for NoScale {
    #[inline(always)]
    fn raw(self) -> u8 {
        0
    }
}

/// Scale fixed at n=1. Used by Cosh, Sinh, Arctanh.
#[derive(Clone, Copy)]
pub struct HyperbolicScale;
impl ScaleValue for HyperbolicScale {
    #[inline(always)]
    fn raw(self) -> u8 {
        1
    }
}

/// Scale n ∈ [0, 7]. Used by Arctangent.
#[derive(Clone, Copy)]
#[repr(u8)]
pub enum AtanScale {
    /// n = 0
    N0 = 0,
    /// n = 1
    N1,
    /// n = 2
    N2,
    /// n = 3
    N3,
    /// n = 4
    N4,
    /// n = 5
    N5,
    /// n = 6
    N6,
    /// n = 7
    N7,
}
impl ScaleValue for AtanScale {
    #[inline(always)]
    fn raw(self) -> u8 {
        self as u8
    }
}

/// Scale n ∈ [1, 4]. Used by natural logarithm.
#[derive(Clone, Copy)]
#[repr(u8)]
pub enum LnScale {
    /// n = 1
    N1 = 1,
    /// n = 2
    N2,
    /// n = 3
    N3,
    /// n = 4
    N4,
}
impl ScaleValue for LnScale {
    #[inline(always)]
    fn raw(self) -> u8 {
        self as u8
    }
}

/// Scale n ∈ [0, 2]. Used by square root.
#[derive(Clone, Copy)]
#[repr(u8)]
pub enum SqrtScale {
    /// n = 0
    N0 = 0,
    /// n = 1
    N1,
    /// n = 2
    N2,
}
impl ScaleValue for SqrtScale {
    #[inline(always)]
    fn raw(self) -> u8 {
        self as u8
    }
}

/// Core trait for CORDIC function types. Implemented by each function ZST.
#[allow(private_bounds)]
pub trait FunctionType: sealed::Sealed {
    /// Hardware function selector
    const FUNC: vals::Func;
    /// Valid scale type for this function
    type Scale: ScaleValue;
    /// True if this function takes two arguments (ARG2 must be written)
    #[doc(hidden)]
    const TWO_ARGS: bool;
    /// True if this function produces two results
    #[doc(hidden)]
    const TWO_RES: bool;
}

/// Function accepts 2 arguments — enables [`Configured::start2`]
pub trait FnTwoArgs: FunctionType {}
/// Function produces 1 result — enables [`Started::result`]
pub trait FnOneRes: FunctionType {}
/// Function produces 2 results — enables [`Started::result2`]
pub trait FnTwoRes: FunctionType {}

macro_rules! define_function {
    (
        $(#[$meta:meta])*
        $name:ident, $func:expr, $scale_ty:ty,
        args: one,
        res: one
    ) => {
        $(#[$meta])*
        pub struct $name;
        impl sealed::Sealed for $name {}
        impl FunctionType for $name {
            const FUNC: vals::Func = $func;
            type Scale = $scale_ty;
            const TWO_ARGS: bool = false;
            const TWO_RES: bool = false;
        }
        impl FnOneRes for $name {}
    };
    (
        $(#[$meta:meta])*
        $name:ident, $func:expr, $scale_ty:ty,
        args: one,
        res: two
    ) => {
        $(#[$meta])*
        pub struct $name;
        impl sealed::Sealed for $name {}
        impl FunctionType for $name {
            const FUNC: vals::Func = $func;
            type Scale = $scale_ty;
            const TWO_ARGS: bool = false;
            const TWO_RES: bool = true;
        }
        impl FnTwoRes for $name {}
    };
    (
        $(#[$meta:meta])*
        $name:ident, $func:expr, $scale_ty:ty,
        args: two,
        res: two
    ) => {
        $(#[$meta])*
        pub struct $name;
        impl sealed::Sealed for $name {}
        impl FunctionType for $name {
            const FUNC: vals::Func = $func;
            type Scale = $scale_ty;
            const TWO_ARGS: bool = true;
            const TWO_RES: bool = true;
        }
        impl FnTwoArgs for $name {}
        impl FnTwoRes for $name {}
    };
}

define_function!(
    /// Cosine function. 2 args (angle, modulus), 2 results (cos, sin).
    Cos, vals::Func::COSINE, NoScale,
    args: two, res: two
);

define_function!(
    /// Sine function. 2 args (angle, modulus), 2 results (sin, cos).
    Sin, vals::Func::SINE, NoScale,
    args: two, res: two
);

define_function!(
    /// Phase (atan2) function. 2 args (x, y), 2 results (phase, modulus).
    Phase, vals::Func::PHASE, NoScale,
    args: two, res: two
);

define_function!(
    /// Modulus function. 2 args (x, y), 2 results (modulus, phase).
    Modulus, vals::Func::MODULUS, NoScale,
    args: two, res: two
);

define_function!(
    /// Arctangent function. 1 arg, 1 result.
    Arctan, vals::Func::ARCTANGENT, AtanScale,
    args: one, res: one
);

define_function!(
    /// Hyperbolic cosine. 1 arg, 2 results (cosh, sinh).
    Cosh, vals::Func::HYPERBOLIC_COSINE, HyperbolicScale,
    args: one, res: two
);

define_function!(
    /// Hyperbolic sine. 1 arg, 2 results (sinh, cosh).
    Sinh, vals::Func::HYPERBOLIC_SINE, HyperbolicScale,
    args: one, res: two
);

define_function!(
    /// Hyperbolic arctangent. 1 arg, 1 result.
    Arctanh, vals::Func::ARCTANH, HyperbolicScale,
    args: one, res: one
);

define_function!(
    /// Natural logarithm. 1 arg, 1 result.
    Ln, vals::Func::NATURAL_LOGARITHM, LnScale,
    args: one, res: one
);

define_function!(
    /// Square root. 1 arg, 1 result.
    Sqrt, vals::Func::SQUARE_ROOT, SqrtScale,
    args: one, res: one
);
