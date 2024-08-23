//! This module just provides the [`integer_power`] function.

use std::ops::Mul;

/// The type used for the `power` parameter in [`integer_power`].
type IntegerPowerType = u16;

/// Calculate `base` to the power of an integer, using the square and multiply algorithm.
pub fn integer_power<T>(base: T, power: IntegerPowerType) -> T
where
    T: Mul<T, Output = T> + PowerZero + std::marker::Copy,
{
    if power == 0 {
        return <T as PowerZero>::POWER_ZERO;
    }

    let mut num = base;

    for bit_idx in (0..power.ilog2()).rev() {
        // Square
        num = num * num;

        if (1 << bit_idx) & power != 0 {
            // Multiply
            num = num * base;
        }
    }

    num
}

pub trait PowerZero {
    /// What is any value of this type raised to the power of 0?
    const POWER_ZERO: Self;
}

impl PowerZero for f32 {
    const POWER_ZERO: Self = 1.0;
}

impl PowerZero for f64 {
    const POWER_ZERO: Self = 1.0;
}

impl PowerZero for glam::DMat2 {
    const POWER_ZERO: Self = glam::DMat2::IDENTITY;
}

impl PowerZero for glam::DMat3 {
    const POWER_ZERO: Self = glam::DMat3::IDENTITY;
}

macro_rules! impl_power_zero_int {
    ($($t:ty),*) => {
        $(impl PowerZero for $t {
            const POWER_ZERO: $t = 1;
        })*
    }
}

impl_power_zero_int!(i8, i16, i32, i64, u8, u16, u32, u64);

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;
    use glam::{DMat2, DMat3, DVec2, DVec3};

    #[test]
    fn integer_power_with_nums() {
        assert_eq!(integer_power(3u64, 15), 3u64.pow(15));
        assert_eq!(integer_power(5u64, 25), 5u64.pow(25));

        // The magnitudes of these answers are huge, so we divide them to better
        // test their approximate equality.
        assert_relative_eq!(
            integer_power(3.21f64, 373) / (3.21f64).powi(373),
            1.,
            epsilon = 0.0000000000001
        );
        assert_relative_eq!(
            integer_power(6.78f64, 123) / (6.78f64).powi(123),
            1.,
            epsilon = 0.0000000000001
        );

        assert_eq!(integer_power(3u64, 1), 3u64);
        assert_eq!(integer_power(3u64, 0), 1u64);
    }

    #[test]
    fn integer_power_with_matrices() {
        let m = DMat2::from_cols(DVec2::new(2.1, -3.2), DVec2::new(0.03, 1.92));
        assert_relative_eq!(integer_power(m, 4), m * m * m * m);

        let n = DMat3::from_cols(
            DVec3::new(2.1, -3.2, 4.5),
            DVec3::new(0.03, 1.92, -1.16),
            DVec3::new(-0.5, 1.34, 7.12),
        );
        assert_relative_eq!(
            integer_power(n, 5),
            n * n * n * n * n,
            epsilon = 0.000000000001
        );

        assert_eq!(integer_power(m, 1), m);
        assert_eq!(integer_power(n, 0), DMat3::IDENTITY);
    }
}
