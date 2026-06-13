use crate::utils::math::{float_eq, float_eq_with};

#[track_caller]
pub fn assert_float_eq(actual: f64, expected: f64) {
    assert!(
        float_eq(actual, expected),
        "assertion failed: {actual} ≉ {expected} (diff {})",
        (actual - expected).abs()
    );
}

#[track_caller]
pub fn assert_float_eq_with(actual: f64, expected: f64, abs_epsilon: f64, rel_epsilon: f64) {
    assert!(
        float_eq_with(actual, expected, abs_epsilon, rel_epsilon),
        "assertion failed: {actual} ≉ {expected} (diff {})",
        (actual - expected).abs()
    );
}
