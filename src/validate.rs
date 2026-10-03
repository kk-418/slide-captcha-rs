/// Checks whether the submitted point `(sx, sy)` matches the target `(dx, dy)`
/// within `padding` pixels on both axes.
///
/// This matches the go-captcha `slide.Validate` algorithm:
/// the accepted window is `[dx - padding, dx + padding]` × `[dy - padding, dy + padding]`.
#[must_use]
pub fn validate(sx: i32, sy: i32, dx: i32, dy: i32, padding: i32) -> bool {
    let new_x = padding * 2;
    let new_y = padding * 2;
    let new_dx = dx - padding;
    let new_dy = dy - padding;
    sx >= new_dx && sx <= new_dx + new_x && sy >= new_dy && sy <= new_dy + new_y
}

#[cfg(test)]
mod tests {
    use super::validate;

    #[test]
    fn padding_zero_requires_exact_match() {
        assert!(validate(10, 20, 10, 20, 0));
        assert!(!validate(11, 20, 10, 20, 0));
        assert!(!validate(10, 21, 10, 20, 0));
        assert!(!validate(9, 20, 10, 20, 0));
        assert!(!validate(10, 19, 10, 20, 0));
    }

    #[test]
    fn padding_five_accepts_plus_minus_five() {
        // window is [5, 15] x [5, 15] when target is (10, 10)
        assert!(validate(10, 10, 10, 10, 5));
        assert!(validate(5, 10, 10, 10, 5));
        assert!(validate(15, 10, 10, 10, 5));
        assert!(validate(10, 5, 10, 10, 5));
        assert!(validate(10, 15, 10, 10, 5));
        assert!(validate(5, 5, 10, 10, 5));
        assert!(validate(15, 15, 10, 10, 5));
    }

    #[test]
    fn padding_five_rejects_outside() {
        assert!(!validate(4, 10, 10, 10, 5));
        assert!(!validate(16, 10, 10, 10, 5));
        assert!(!validate(10, 4, 10, 10, 5));
        assert!(!validate(10, 16, 10, 10, 5));
        assert!(!validate(4, 4, 10, 10, 5));
        assert!(!validate(16, 16, 10, 10, 5));
    }

    #[test]
    fn both_axes_must_match() {
        assert!(!validate(10, 100, 10, 10, 5));
        assert!(!validate(100, 10, 10, 10, 5));
        assert!(validate(12, 8, 10, 10, 5));
    }
}
