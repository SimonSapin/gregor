use core::ops::{Div, Rem, Add, Sub};

pub trait Integer: Sized + Copy + Ord + Div<Output=Self> + Rem<Output=Self> +
                   Add<Output=Self> + Sub<Output=Self> {
    fn zero() -> Self;
    fn one() -> Self;
}

impl Integer for i32 {
    fn zero() -> Self { 0 }
    fn one() -> Self { 1 }
}

impl Integer for i64 {
    fn zero() -> Self { 0 }
    fn one() -> Self { 1 }
}

/// Remainder within range 0..divisor, even for negative dividend
pub fn positive_rem<Int: Integer>(dividend: Int, divisor: Int) -> Int {
    let rem = dividend % divisor;
    if rem < Int::zero() {
        rem + divisor
    } else {
        rem
    }
}

/// Integer divison that rounds towards negative infinity
pub fn div_floor<Int: Integer>(dividend: Int, divisor: Int) -> Int {
    if dividend > Int::zero() {
        dividend / divisor
    } else {
        (dividend + Int::one() - divisor) / divisor
    }
}
