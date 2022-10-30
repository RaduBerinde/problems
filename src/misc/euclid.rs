fn euclid(a: i64, b: i64) -> (i64, i64, i64) {
    if a == b {
        return (1, 0, a);
    }
    if a == 0 {
        return (0, 1, b);
    }
    let (x, y, d) = euclid(b % a, a);
    // d = (b % a) * x + a * y
    //   = (b - (b/a)*a) * x + a * y
    //   = a * (y - (b/a)*x) + b * x
    (y - (b / a) * x, x, d)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_euclid() {
        assert_eq!(euclid(5, 3), (-1, 2, 1));
        assert_eq!(euclid(10, 4), (1, -2, 2));
        assert_eq!(euclid(10, 10), (1, 0, 10));
    }
}
