/// Input value is out of range for Q1.x format
#[allow(missing_docs)]
#[derive(Debug)]
pub enum NumberOutOfRange {
    BelowLowerBound,
    AboveUpperBound,
}

#[cfg(feature = "defmt")]
impl defmt::Format for NumberOutOfRange {
    fn format(&self, fmt: defmt::Formatter) {
        match self {
            NumberOutOfRange::BelowLowerBound => defmt::write!(fmt, "input value should be >= -1"),
            NumberOutOfRange::AboveUpperBound => defmt::write!(fmt, "input value should be <= 1"),
        }
    }
}
