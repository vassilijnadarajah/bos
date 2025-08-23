// SPDX-License-Identifier: GPL-3.0-only
// (C) 2025 Vassilij Nadarajah
// packages@vassilijnadarajah.de

use crate::point2d::{AsPoint2D, Point2D};

pub fn calc_line_polygon_overlay<T: AsPoint2D + Copy>(line: &Vec<T>, polygon: &Vec<Option<Point2D>>) -> f64 {
    // ToDo
    0.0
}

/// Calculates the total length of a polyline
///
/// ## Parameters
/// - `line`: Reference to a vector of `Point2D` representing the polyline.
///
/// ## Returns
/// The total length of the polyline.
pub fn calc_line_length<T: AsPoint2D + Copy>(line: &Vec<T>) -> f64 {
    let mut length = 0.0;
    for i in 1..line.len() {
        length += (line[i].to_point2d() - line[i - 1].to_point2d()).abs();
    }
    length
}

/// Creates a buffer polygon around a polyline represented by a vector of 2D points.
///
/// ## Parameters
/// - `line`: Reference to a vector of `Point2D` representing the polyline.
/// - `buffer`: The buffer distance (width) to offset from the line.
///
/// ## Returns
/// Returns a vector of `Point2D` representing the buffered polygon around the input line.
/// If the input line has fewer than two points, returns an empty vector.
///
/// ## Edge Cases
/// - If `line` has less than two points, no buffer is created and `None` is returned.
/// - Handles parallel segments by using helper points for intersection.
///
/// ## Examples
/// ```
/// let line = vec![Point2D::new(0.0, 0.0), Point2D::new(1.0, 1.0)];
/// let buffer = 0.5;
/// let buffered = create_line_buffer(&line, buffer);
/// ```
pub fn create_line_buffer<T: AsPoint2D + Copy>(line: &Vec<T>, buffer: f64) -> Option<Vec<Option<Point2D>>> {
    let mut buffered_line: Vec<Option<Point2D>> = vec![None; 2 * line.len()];
    if line.len() < 2 {
        return None; // Not enough points to create a buffer
    }
    for i in 0..line.len() {
        let current = &line[i].to_point2d();
        let next = if i < line.len() - 1 { Some(line[i + 1].to_point2d()) } else { None };
        let previous = if i > 0 { Some(line[i - 1].to_point2d()) } else { None };
        let last_index = buffered_line.len() - 1;

        if previous.is_none() {
            let new_point_right = get_orthogonal_point2d(current, &next.unwrap(), buffer);
            let new_point_left = *current + (*current - new_point_right);
            buffered_line[i] = Some(new_point_right);
            buffered_line[last_index - i] = Some(new_point_left);
        } else if next.is_none() {
            let new_point_right = get_orthogonal_point2d(current, &previous.unwrap(), -buffer);
            let new_point_left = *current + (*current - new_point_right);
            buffered_line[i] = Some(new_point_right);
            buffered_line[last_index - i] = Some(new_point_left);
        } else {
            // Get helper points
            let h1 = get_orthogonal_point2d(&previous.unwrap(), current, buffer);
            let h2 = h1 + (*current - previous.unwrap());
            let h3 = get_orthogonal_point2d(current, &next.unwrap(), buffer);
            let h4 = h3 + (next.unwrap() - *current);

            // Calc intersection
            let intersection = calc_line_intersection(&h1, &h2, &h3, &h4);
            if let Some(intersection_point) = intersection {
                buffered_line[i] = Some(intersection_point);
            } else {
                buffered_line[i] = Some(h2); // lines are parallel, hence use h2
            }

            buffered_line[last_index - i] = Some(*current - buffered_line[i].unwrap() + *current);
        }
    }

    buffered_line.push(buffered_line[0]); // Close the polygon

    Some(buffered_line)
}

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
    use pyo3::buffer;

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

    #[test]
    fn test_create_line_buffer_simple_line() {
        let line = vec![Point2D::new(0.0, 0.0), Point2D::new(1.0, 1.0)];
        let buffer = 0.5;
        let buffered_line = create_line_buffer(&line, buffer).unwrap();
        let a = (buffer.powi(2) / 2.0).sqrt();
        assert_eq!(buffered_line.len(), 5);
        assert!((buffered_line[0].unwrap().x - a).abs() < 1e-10);
        assert!((buffered_line[0].unwrap().y - (-a)).abs() < 1e-10);
        assert!((buffered_line[1].unwrap().x - (1.0 + a)).abs() < 1e-10);
        assert!((buffered_line[1].unwrap().y - (1.0 - a)).abs() < 1e-10);
        assert!((buffered_line[2].unwrap().x - (1.0 - a)).abs() < 1e-10);
        assert!((buffered_line[2].unwrap().y - (1.0 + a)).abs() < 1e-10);
        assert!((buffered_line[3].unwrap().x - (-a)).abs() < 1e-10);
        assert!((buffered_line[3].unwrap().y - a).abs() < 1e-10);
        assert!((buffered_line[4].unwrap().x - a).abs() < 1e-10);
        assert!((buffered_line[4].unwrap().y - (-a)).abs() < 1e-10);
    }

    #[test]
    fn test_create_line_buffer_polyline() {
        let line = vec![
            Point2D::new(0.0, 0.0),
            Point2D::new(0.0, 1.0),
            Point2D::new(2.0, 1.0),
        ];
        let buffer = 0.2;
        let buffered_line = create_line_buffer(&line, buffer).unwrap();
        assert_eq!(buffered_line.len(), 7);
        let a = (buffer.powi(2) / 2.0).sqrt();
        assert!((buffered_line[0].unwrap().x - buffer).abs() < 1e-10);
        assert!((buffered_line[0].unwrap().y).abs() < 1e-10);
        assert!((buffered_line[1].unwrap().x - buffer).abs() < 1e-10);
        assert!((buffered_line[1].unwrap().y - (1.0 - buffer)).abs() < 1e-10);
        assert!((buffered_line[2].unwrap().x - 2.0).abs() < 1e-10);
        assert!((buffered_line[2].unwrap().y - (1.0 - buffer)).abs() < 1e-10);
        assert!((buffered_line[3].unwrap().x - 2.0).abs() < 1e-10);
        assert!((buffered_line[3].unwrap().y - (1.0 + buffer)).abs() < 1e-10);
        assert!((buffered_line[4].unwrap().x - (-buffer)).abs() < 1e-10);
        assert!((buffered_line[4].unwrap().y - (1.0 + buffer)).abs() < 1e-10);
        assert!((buffered_line[5].unwrap().x - (-buffer)).abs() < 1e-10);
        assert!((buffered_line[5].unwrap().y).abs() < 1e-10);
        assert!((buffered_line[6].unwrap().x - buffer).abs() < 1e-10);
        assert!((buffered_line[6].unwrap().y).abs() < 1e-10);
    }

    #[test]
    fn test_calc_line_length() {
        let line = vec![
            Point2D::new(-3.0, 4.0),
            Point2D::new(0.0, 0.0),
            Point2D::new(6.0, 8.0),
        ];
        let length = calc_line_length(&line);
        assert!((length - 15.0).abs() < 1e-10);
    }
}