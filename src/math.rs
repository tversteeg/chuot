//! Simple math functions.

/// Linearly interpolate between two values.
///
/// Internally this calls:
///
/// ```
/// # fn func(lhs: f32, rhs: f32, factor: f32) -> f32{
/// lhs + (rhs - lhs) * factor
/// # }
/// ```
///
/// # Arguments
///
/// * `lhs` - First value, will be returned if `factor == 0.0`.
/// * `rhs` - Second value, will be returned if `factor == 1.0`.
/// * `factor` - Interpolation value, when `factor == 0.0`, `lhs` will be returned, when `factor == 1.0`, `rhs` will be returned.
///
/// # Returns
///
/// - A random number between `min` and `max`.
#[inline]
#[must_use]
pub fn lerp(lhs: f32, rhs: f32, factor: f32) -> f32 {
    (rhs - lhs).mul_add(factor, lhs)
}

#[cfg(test)]
mod tests {
    use glam::FloatExt;

    #[test]
    fn lerp() {
        assert!((super::lerp(5.0, 10.0, 0.5) - 5.0_f32.lerp(10.0, 0.5)).abs() < f32::EPSILON);
    }
}
