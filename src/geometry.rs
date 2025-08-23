// SPDX-License-Identifier: GPL-3.0-only
// (C) 2025 Vassilij Nadarajah
// packages@vassilijnadarajah.de

use crate::point2d::{AsPoint2D, Point2D};

/// Get the orthogonal point to the line segment (p1, p2) at a given buffer width.
fn get_orthogonal_point2d(p1: &Point2D, p2: &Point2D, buffer: f64) -> Point2D {
    let direction = *p2 - *p1;
    let length = direction.abs();
    if length == 0.0 {
        return *p1;
    }
    let ortho = Point2D::new(direction.y / length * buffer, -direction.x / length * buffer);
    return *p1 + ortho;
}

/// Calculate the intersection point of two infinite lines (a1, a2) and (b1, b2).
/// Returns the intersection point as a Point2D, or None if lines are parallel.
fn calc_line_intersection(a1: &Point2D, a2: &Point2D, b1: &Point2D, b2: &Point2D) -> Option<Point2D> {
    // Direction vectors
    let da = *a2 - *a1;
    let db = *b2 - *b1;
    let denom = (da.x) * (db.y) - (db.x) * (da.y);
    if denom == 0.0 {
        return None; // Lines are parallel
    }
    // Compute intersection using determinants
    // Line A: a1 + t * da
    // Line B: b1 + s * db
    // Find t where they intersect
    let delta = *b1 - *a1;
    let t = (delta.x * db.y - delta.y * db.x) / denom;
    Some(Point2D::new(a1.x + t * da.x, a1.y + t * da.y))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn calc_line_intersection_two_regular_lines() {
        let a1 = Point2D::new(1.0, 1.0);
        let a2 = Point2D::new(4.0, 4.0);
        let b1 = Point2D::new(1.0, 4.0);
        let b2 = Point2D::new(4.0, 1.0);

        let intersection = calc_line_intersection(&a1, &a2, &b1, &b2);
        assert_eq!(intersection, Some(Point2D::new(2.5, 2.5)));
    }

    #[test]
    fn calc_line_intersection_parallel_lines() {
        let a1 = Point2D::new(1.0, 1.0);
        let a2 = Point2D::new(4.0, 4.0);
        let b1 = Point2D::new(1.0, 2.0);
        let b2 = Point2D::new(4.0, 5.0);

        let intersection = calc_line_intersection(&a1, &a2, &b1, &b2);
        assert_eq!(intersection, None); // Lines are parallel
    }

    #[test]
    fn calc_line_intersection_outside_segments() {
        let a1 = Point2D::new(1, 1);
        let a2 = Point2D::new(4, 4);
        let b1 = Point2D::new(0, 5);
        let b2 = Point2D::new(0, 2);

        let intersection = calc_line_intersection(&a1, &a2, &b1, &b2);
        assert_eq!(intersection, Some(Point2D::new(0, 0)));
    }

    #[test]
    fn test_get_orthogonal_point2d() {
        let p1 = Point2D::new(0.0, 0.0);
        let p2 = Point2D::new(2.0, 2.0);
        let buffer = 0.5;

        let ortho_point = get_orthogonal_point2d(&p1, &p2, buffer);
        let expected_point = Point2D::new((buffer.powi(2) / 2.0).sqrt(), -(buffer.powi(2) / 2.0).sqrt());
        assert!((ortho_point.x - expected_point.x).abs() < 1e-10);
        assert!((ortho_point.y - expected_point.y).abs() < 1e-10);
    }
}