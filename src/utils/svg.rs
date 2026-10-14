//! Geometry helpers for hand-drawn SVG charts.

use std::fmt::Write;

/// A point in chart space.
pub type Point = (f64, f64);

/// Downsamples `data` to at most `threshold` points with Largest-Triangle-Three-Buckets,
/// which keeps the peaks and troughs that make a price trace recognisable.
pub fn lttb(data: &[Point], threshold: usize) -> Vec<Point> {
    let n = data.len();
    if threshold >= n || threshold < 3 {
        return data.to_vec();
    }

    // `threshold < n` guarantees every bucket is non-empty.
    let buckets = threshold - 2;
    let bucket_start = |bucket: usize| (1 + bucket * (n - 2) / buckets).min(n);

    let mut sampled = Vec::with_capacity(threshold);
    sampled.push(data[0]);
    let mut previous = data[0];

    for bucket in 0..buckets {
        let (avg_x, avg_y) = mean(&data[bucket_start(bucket + 1)..bucket_start(bucket + 2)]);

        let candidates = &data[bucket_start(bucket)..bucket_start(bucket + 1)];
        let chosen = candidates
            .iter()
            .copied()
            .max_by(|&a, &b| {
                triangle_area(previous, a, (avg_x, avg_y)).total_cmp(&triangle_area(
                    previous,
                    b,
                    (avg_x, avg_y),
                ))
            })
            .unwrap_or(previous);

        sampled.push(chosen);
        previous = chosen;
    }

    sampled.push(data[n - 1]);
    sampled
}

fn mean(points: &[Point]) -> Point {
    let count = points.len().max(1) as f64;
    let (sum_x, sum_y) = points
        .iter()
        .fold((0.0, 0.0), |(sx, sy), (x, y)| (sx + x, sy + y));
    (sum_x / count, sum_y / count)
}

fn triangle_area(a: Point, b: Point, c: Point) -> f64 {
    (a.0 - c.0)
        .mul_add(b.1 - a.1, -((a.0 - b.0) * (c.1 - a.1)))
        .abs()
}

/// Maps `points` into a `width × height` box, leaving `padding` above and below.
/// Larger values are drawn higher; a flat series is centred vertically.
pub fn fit_to_box(points: &[Point], width: f64, height: f64, padding: f64) -> Vec<Point> {
    let (Some(first), Some(last)) = (points.first(), points.last()) else {
        return Vec::new();
    };
    let (min_y, max_y) = points
        .iter()
        .fold((f64::INFINITY, f64::NEG_INFINITY), |(lo, hi), &(_, y)| {
            (lo.min(y), hi.max(y))
        });
    let x_span = last.0 - first.0;
    let y_span = max_y - min_y;
    let inner_height = 2.0f64.mul_add(-padding, height);

    points
        .iter()
        .map(|&(x, y)| {
            let x = if x_span > 0.0 {
                (x - first.0) / x_span * width
            } else {
                width / 2.0
            };
            let y = if y_span > f64::EPSILON {
                ((max_y - y) / y_span).mul_add(inner_height, padding)
            } else {
                height / 2.0
            };
            (x, y)
        })
        .collect()
}

/// SVG path data joining `points` with smooth Catmull-Rom curves.
pub fn smooth_path(points: &[Point]) -> String {
    let Some(&(x, y)) = points.first() else {
        return String::new();
    };
    let mut path = format!("M{x:.1},{y:.1}");
    for i in 0..points.len().saturating_sub(1) {
        let p0 = points[i.saturating_sub(1)];
        let p1 = points[i];
        let p2 = points[i + 1];
        let p3 = points.get(i + 2).copied().unwrap_or(p2);

        // Catmull-Rom tangents converted to cubic Bézier control points.
        let tension = 6.0;
        let c1 = (
            p1.0 + (p2.0 - p0.0) / tension,
            p1.1 + (p2.1 - p0.1) / tension,
        );
        let c2 = (
            p2.0 - (p3.0 - p1.0) / tension,
            p2.1 - (p3.1 - p1.1) / tension,
        );
        let _ = write!(
            path,
            "C{:.1},{:.1} {:.1},{:.1} {:.1},{:.1}",
            c1.0, c1.1, c2.0, c2.1, p2.0, p2.1
        );
    }
    path
}

#[cfg(test)]
mod tests {
    use super::*;

    fn series(values: &[f64]) -> Vec<Point> {
        values
            .iter()
            .enumerate()
            .map(|(i, &v)| (i as f64, v))
            .collect()
    }

    #[test]
    fn lttb_returns_short_input_unchanged() {
        let data = series(&[1.0, 5.0, 2.0]);
        assert_eq!(lttb(&data, 10), data);
    }

    #[test]
    fn lttb_keeps_endpoints_and_spikes() {
        let mut values = vec![10.0; 100];
        values[37] = 90.0;
        values[71] = -5.0;
        let data = series(&values);

        let sampled = lttb(&data, 10);

        assert_eq!(sampled.len(), 10);
        assert_eq!(sampled.first(), data.first());
        assert_eq!(sampled.last(), data.last());
        assert!(sampled.contains(&(37.0, 90.0)));
        assert!(sampled.contains(&(71.0, -5.0)));
        assert!(sampled.windows(2).all(|pair| pair[0].0 < pair[1].0));
    }

    #[test]
    fn fit_to_box_maps_extremes_to_padding() {
        let fitted = fit_to_box(&series(&[0.0, 10.0]), 100.0, 50.0, 5.0);
        assert_eq!(fitted, vec![(0.0, 45.0), (100.0, 5.0)]);
    }

    #[test]
    fn fit_to_box_centres_a_flat_line() {
        let fitted = fit_to_box(&series(&[3.0, 3.0]), 100.0, 50.0, 5.0);
        assert_eq!(fitted, vec![(0.0, 25.0), (100.0, 25.0)]);
    }

    #[test]
    fn smooth_path_has_one_curve_per_segment() {
        let path = smooth_path(&[(0.0, 0.0), (1.0, 1.0), (2.0, 0.0)]);
        assert!(path.starts_with("M0.0,0.0"));
        assert_eq!(path.matches('C').count(), 2);
        assert!(smooth_path(&[]).is_empty());
    }
}
