//! Generate random numbers.

/// Generate a random number between the range.
///
/// Internally this calls:
///
/// ```
/// # let (min, max) = (0.0, 0.0);
/// fastrand::f32().mul_add(max - min, min);
/// ```
///
/// # Arguments
///
/// * `min` - Start of the random value, must be smaller than `max`.
/// * `max` - End of the random value, must be bigger than `min`.
///
/// # Returns
///
/// - A random number between `min` and `max`.
#[inline]
#[must_use]
pub fn random(min: f32, max: f32) -> f32 {
    fastrand::f32().mul_add(max - min, min)
}
