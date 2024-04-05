//! Generate random numbers.

/// Generate a random number between 0.0 and 1.0.
///
/// # Returns
///
/// - A random number between 0.0 and 1.0.
pub fn random() -> f32 {
    fastrand::f32()
}

/// Generate a random number between the range.
///
/// # Arguments
///
/// * `min` - Start of the random value, must be smaller than `max`.
/// * `max` - End of the random value, must be bigger than `max`.
///
/// # Returns
///
/// - A random number between `min` and `max`.
pub fn random_range(min: f32, max: f32) -> f32 {
    fastrand::f32() * (max - min) + min
}
