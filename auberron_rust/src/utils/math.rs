pub const ABS_EPSILON: f64 = 1e-9f64;
pub const REL_EPSILON: f64 = 1e-5f64;

pub fn float_eq(a: f64, b: f64) -> bool {
    let delta = (a - b).abs();
    (delta < ABS_EPSILON) || (delta < (a.abs().max(b.abs()) * REL_EPSILON))
}

pub fn float_eq_with(a: f64, b: f64, abs_epsilon: f64, rel_epsilon: f64) -> bool {
    let delta = (a - b).abs();
    (delta < abs_epsilon) || (delta < (a.abs().max(b.abs()) * rel_epsilon))
}
