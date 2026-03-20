/// CORDIC precision (number of iterations)
#[allow(missing_docs)]
#[derive(Debug, Clone, Copy, Default)]
pub enum Precision {
    Iters4 = 1,
    Iters8,
    Iters12,
    Iters16,
    Iters20,
    #[default]
    Iters24, // this value is recommended by Reference Manual
    Iters28,
    Iters32,
    Iters36,
    Iters40,
    Iters44,
    Iters48,
    Iters52,
    Iters56,
    Iters60,
}
