//! The few summary statistics the insights share.

pub(super) fn mean(values: &[f64]) -> Option<f64> {
    if values.is_empty() {
        return None;
    }
    Some(values.iter().sum::<f64>() / values.len() as f64)
}

/// The sample standard deviation, which needs at least two values.
pub(super) fn standard_deviation(values: &[f64]) -> Option<f64> {
    if values.len() < 2 {
        return None;
    }
    let mean = mean(values)?;
    let squares: f64 = values.iter().map(|value| (value - mean).powi(2)).sum();
    Some((squares / (values.len() - 1) as f64).sqrt())
}

pub(super) fn median(values: &[f64]) -> Option<f64> {
    if values.is_empty() {
        return None;
    }
    let mut sorted = values.to_vec();
    sorted.sort_by(f64::total_cmp);
    let middle = sorted.len() / 2;
    Some(if sorted.len().is_multiple_of(2) {
        (sorted[middle - 1] + sorted[middle]) / 2.0
    } else {
        sorted[middle]
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn summarises_a_small_sample() {
        let values = [2.0, 4.0, 4.0, 4.0, 5.0, 5.0, 7.0, 9.0];

        assert_eq!(mean(&values), Some(5.0));
        assert_eq!(median(&values), Some(4.5));
        let deviation = standard_deviation(&values).unwrap();
        assert!((deviation - 2.138).abs() < 1e-3, "was {deviation}");
    }

    #[test]
    fn has_nothing_to_say_about_too_little() {
        assert_eq!(mean(&[]), None);
        assert_eq!(median(&[]), None);
        assert_eq!(standard_deviation(&[1.0]), None);
    }
}
