pub(crate) fn stats(values: &[f64]) -> (f64, f64, f64, f64) {
    if values.is_empty() {
        return (0.0, 0.0, 0.0, 0.0);
    }
    let mut min = f64::INFINITY;
    let mut max = f64::NEG_INFINITY;
    let mut sum = 0.0;
    for value in values {
        min = min.min(*value);
        max = max.max(*value);
        sum += value;
    }
    let mean = sum / values.len() as f64;
    let variance = values
        .iter()
        .map(|value| {
            let delta = *value - mean;
            delta * delta
        })
        .sum::<f64>()
        / values.len() as f64;
    (mean, variance.sqrt(), min, max)
}

pub(crate) fn zero_crossings_along_x(field: &[f64], n: usize) -> f64 {
    if n < 2 {
        return 0.0;
    }
    let mut crossings = 0usize;
    for row in 0..n {
        for col in 0..(n - 1) {
            let here = field[row * n + col] >= 0.0;
            let next = field[row * n + col + 1] >= 0.0;
            if here != next {
                crossings += 1;
            }
        }
    }
    crossings as f64 / n as f64
}

pub(crate) fn zero_crossings_along_y(field: &[f64], n: usize) -> f64 {
    if n < 2 {
        return 0.0;
    }
    let mut crossings = 0usize;
    for row in 0..(n - 1) {
        for col in 0..n {
            let here = field[row * n + col] >= 0.0;
            let next = field[(row + 1) * n + col] >= 0.0;
            if here != next {
                crossings += 1;
            }
        }
    }
    crossings as f64 / n as f64
}

pub(crate) fn correlation(left: &[f64], right: &[f64]) -> f64 {
    if left.is_empty() || left.len() != right.len() {
        return 0.0;
    }
    let (left_mean, left_std, _, _) = stats(left);
    let (right_mean, right_std, _, _) = stats(right);
    let denom = (left_std * right_std).max(1.0e-12);
    left.iter()
        .zip(right.iter())
        .map(|(l, r)| (*l - left_mean) * (*r - right_mean))
        .sum::<f64>()
        / (left.len() as f64 * denom)
}
