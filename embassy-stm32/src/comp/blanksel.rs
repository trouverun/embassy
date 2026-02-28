#![allow(missing_docs)]

macro_rules! blanksel_enum {
    ($name:ident, $($variant:ident = $val:expr),+ $(,)?) => {
        #[derive(Debug, Copy, Clone, Eq, PartialEq)]
        #[cfg_attr(feature = "defmt", derive(defmt::Format))]
        pub enum $name {
            None = 0b000,
            $($variant = $val,)+
        }

        impl From<$name> for u8 {
            fn from(val: $name) -> u8 {
                val as u8
            }
        }
    };
}

#[cfg(stm32g4)]
blanksel_enum!(Comp1BlankSel,
    Tim1Oc5  = 0b001,
    Tim2Oc3  = 0b010,
    Tim3Oc3  = 0b011,
    Tim8Oc5  = 0b100,
    Tim20Oc5 = 0b101,
    Tim15Oc1 = 0b110,
    Tim4Oc3  = 0b111,
);

#[cfg(stm32g4)]
blanksel_enum!(Comp2BlankSel,
    Tim1Oc5  = 0b001,
    Tim2Oc3  = 0b010,
    Tim3Oc3  = 0b011,
    Tim8Oc5  = 0b100,
    Tim20Oc5 = 0b101,
    Tim15Oc1 = 0b110,
    Tim4Oc3  = 0b111,
);

#[cfg(stm32g4)]
blanksel_enum!(Comp3BlankSel,
    Tim1Oc5  = 0b001,
    Tim3Oc3  = 0b010,
    Tim2Oc4  = 0b011,
    Tim8Oc5  = 0b100,
    Tim20Oc5 = 0b101,
    Tim15Oc1 = 0b110,
    Tim4Oc3  = 0b111,
);

#[cfg(stm32g4)]
blanksel_enum!(Comp4BlankSel,
    Tim3Oc4  = 0b001,
    Tim8Oc5  = 0b010,
    Tim15Oc1 = 0b011,
    Tim1Oc5  = 0b100,
    Tim20Oc5 = 0b101,
    Tim4Oc3  = 0b111,
);

#[cfg(stm32g4)]
blanksel_enum!(Comp5BlankSel,
    Tim2Oc3  = 0b001,
    Tim8Oc5  = 0b010,
    Tim3Oc3  = 0b011,
    Tim1Oc5  = 0b100,
    Tim20Oc5 = 0b101,
    Tim15Oc1 = 0b110,
    Tim4Oc3  = 0b111,
);

#[cfg(stm32g4)]
blanksel_enum!(Comp6BlankSel,
    Tim8Oc5  = 0b001,
    Tim2Oc4  = 0b010,
    Tim15Oc2 = 0b011,
    Tim1Oc5  = 0b100,
    Tim20Oc5 = 0b101,
    Tim15Oc1 = 0b110,
    Tim4Oc3  = 0b111,
);

#[cfg(stm32g4)]
blanksel_enum!(Comp7BlankSel,
    Tim1Oc5  = 0b001,
    Tim8Oc5  = 0b010,
    Tim3Oc3  = 0b011,
    Tim15Oc2 = 0b100,
    Tim20Oc5 = 0b101,
    Tim15Oc1 = 0b110,
    Tim4Oc3  = 0b111,
);
