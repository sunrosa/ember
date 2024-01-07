pub(crate) fn weighted_mean(data: Vec<(f64, f64)>) -> f64 {
    let mut sum = 0.0;
    let mut weighting_factor_sum = 0.0;

    for point in data {
        sum += point.0 * point.1;
        weighting_factor_sum += point.1;
    }

    sum / weighting_factor_sum
}
