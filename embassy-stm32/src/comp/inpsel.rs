#![allow(missing_docs)]

macro_rules! inpsel_enum {
    ($name:ident, $pin0:ident, $pin1:ident) => {
        #[derive(Debug, Copy, Clone, Eq, PartialEq)]
        #[cfg_attr(feature = "defmt", derive(defmt::Format))]
        pub enum $name {
            $pin0,
            $pin1,
        }

        impl From<$name> for bool {
            fn from(val: $name) -> bool {
                match val {
                    $name::$pin0 => false,
                    $name::$pin1 => true,
                }
            }
        }
    };
}

#[cfg(stm32g4)]
inpsel_enum!(Comp1InpSel, PA1, PB1);
#[cfg(stm32g4)]
inpsel_enum!(Comp2InpSel, PA7, PA3);
#[cfg(stm32g4)]
inpsel_enum!(Comp3InpSel, PA0, PC1);
#[cfg(stm32g4)]
inpsel_enum!(Comp4InpSel, PB0, PE7);
#[cfg(stm32g4)]
inpsel_enum!(Comp5InpSel, PB13, PD12);
#[cfg(stm32g4)]
inpsel_enum!(Comp6InpSel, PB11, PD11);
#[cfg(stm32g4)]
inpsel_enum!(Comp7InpSel, PB14, PD14);
