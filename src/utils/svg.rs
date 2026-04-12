/// Normalizes a slice of values into SVG coordinate points.
///
/// Maps each value to an (x, y) coordinate within the given `width`,
/// `height`, and `padding` bounds. Returns `(points, min, max, range)`
/// so callers can reuse the computed bounds.
pub fn normalize_points(
    values: &[f64],
    width: f64,
    height: f64,
    padding: f64,
) -> (Vec<(f64, f64)>, f64, f64, f64) {
    if values.is_empty() {
        return (vec![], 0.0, 0.0, 1.0);
    }

    let min = values.iter().copied().fold(f64::INFINITY, f64::min);
    let max = values.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    let range = if (max - min).abs() < 0.01 {
        1.0 // Avoid division by zero for flat lines (threshold: 0.01p)
    } else {
        max - min
    };

    let points: Vec<(f64, f64)> = values
        .iter()
        .enumerate()
        .map(|(i, &val)| {
            let x = if values.len() > 1 {
                (i as f64 / (values.len() - 1) as f64) * width
            } else {
                width / 2.0 // Centre single point
            };
            let y = (1.0 - (val - min) / range).mul_add(2.0f64.mul_add(-padding, height), padding);
            (x, y)
        })
        .collect();

    (points, min, max, range)
}
