#![allow(missing_docs)]

macro_rules! inmsel_enum {
    ($name:ident, $dac0:ident, $dac1:ident, $pin0:ident, $pin1:ident) => {
        #[derive(Debug, Copy, Clone, Eq, PartialEq)]
        #[cfg_attr(feature = "defmt", derive(defmt::Format))]
        pub enum $name {
            /// 1/4 VREFINT
            Vref1_4 = 0b000,
            /// 1/2 VREFINT
            Vref1_2 = 0b001,
            /// 3/4 VREFINT
            Vref3_4 = 0b010,
            /// VREFINT
            Vref = 0b011,
            $dac0 = 0b100,
            $dac1 = 0b101,
            $pin0 = 0b110,
            $pin1 = 0b111,
        }

        impl From<$name> for u8 {
            fn from(val: $name) -> u8 {
                val as u8
            }
        }
    };
}

#[cfg(stm32g4)]
inmsel_enum!(Comp1InmSel, Dac3Ch1, Dac1Ch1, PA4, PA0);
#[cfg(stm32g4)]
inmsel_enum!(Comp2InmSel, Dac3Ch2, Dac1Ch2, PA5, PA2);
#[cfg(stm32g4)]
inmsel_enum!(Comp3InmSel, Dac3Ch1, Dac1Ch1, PF1, PC0);
#[cfg(stm32g4)]
inmsel_enum!(Comp4InmSel, Dac3Ch2, Dac1Ch1, PE8, PB2);
#[cfg(stm32g4)]
inmsel_enum!(Comp5InmSel, Dac4Ch1, Dac1Ch2, PB10, PD13);
#[cfg(stm32g4)]
inmsel_enum!(Comp6InmSel, Dac4Ch2, Dac2Ch1, PD10, PB15);
#[cfg(stm32g4)]
inmsel_enum!(Comp7InmSel, Dac4Ch1, Dac2Ch1, PD15, PB12);
