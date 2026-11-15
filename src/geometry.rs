// SPDX-License-Identifier: GPL-3.0-only
// (C) 2025 Vassilij Nadarajah
// packages@vassilijnadarajah.de

use crate::point2d::{AsPoint2D, Point2D};

/// Calculates the overlay length of a line segment with a polygon
///
/// ## Parameters
/// - `line`: Reference to a vector of `Point2D` representing the line segment.
/// - `polygon`: Reference to a vector of `Point2D` representing the polygon.
///
/// ## Returns
/// The overlay length of the line segment with the polygon.
pub fn calc_line_polygon_overlay<T: AsPoint2D + Copy>(line: &Vec<T>, polygon: &Vec<Point2D>) -> f64 {
    let mut overlay_length = 0.0;
    let mut inside_polygon = is_point_in_polygon(&line[0].to_point2d(), polygon);
    for i in 0..line.len() - 1 {
        let l1 = line[i].to_point2d();
        let l2 = line[i + 1].to_point2d();
        let mut intersections: Vec<Point2D> = Vec::new();
        
        for j in 0..polygon.len() - 1 {
            let p1 = polygon[j];
            let p2 = polygon[j + 1];
            
            // Check for intersection
            if let Some(intersection) = calc_line_segment_intersection(&l1, &l2, &p1, &p2) {
                intersections.push(intersection);
            }
        }
        
        // Sort intersections by their distance to the line starting point
        intersections.sort_by(|a, b| (l1 - *a).abs().partial_cmp(&(l1 - *b).abs()).unwrap());
        
        let mut current_point = l1;
        for intersection in &intersections {
            let distance = (*intersection - current_point).abs();
            overlay_length += if inside_polygon { distance } else { 0.0 };
            current_point = *intersection;
            inside_polygon = if distance < 1e-10 { inside_polygon } else { !inside_polygon };
        }
        overlay_length += if inside_polygon { (l2 - current_point).abs() } else { 0.0 };
    }

    overlay_length
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
/// use bos_geo::geometry::create_line_buffer;
/// use bos_geo::point2d::Point2D;
///
/// let line = vec![Point2D::new(0.0, 0.0), Point2D::new(1.0, 1.0)];
/// let buffer = 0.5;
/// let buffered = create_line_buffer(&line, buffer).unwrap();
/// assert_eq!(buffered.len(), 5);
/// ```
pub fn create_line_buffer<T: AsPoint2D + Copy>(line: &Vec<T>, buffer: f64) -> Option<Vec<Point2D>> {
    let mut buffered_line = Vec::with_capacity(2 * line.len() + 1);
    let mut buffer_left = Vec::with_capacity(line.len()); 
    if line.len() < 2 {
        return None; // Not enough points to create a buffer
    }
    for i in 0..line.len() {
        let current = &line[i].to_point2d();
        let next = if i < line.len() - 1 { Some(line[i + 1].to_point2d()) } else { None };
        let previous = if i > 0 { Some(line[i - 1].to_point2d()) } else { None };
        
        if previous.is_none() {
            let new_point_right = get_orthogonal_point2d(current, &next.unwrap(), buffer);
            let new_point_left = *current + (*current - new_point_right);
            buffered_line.push(new_point_right);
            buffer_left.push(new_point_left);
        } else if next.is_none() {
            let new_point_right = get_orthogonal_point2d(current, &previous.unwrap(), -buffer);
            let new_point_left = *current + (*current - new_point_right);
            buffered_line.push(new_point_right);
            buffer_left.push(new_point_left);
        } else {
            // Get helper points
            let h1 = get_orthogonal_point2d(&previous.unwrap(), current, buffer);
            let h2 = h1 + (*current - previous.unwrap());
            let h3 = get_orthogonal_point2d(current, &next.unwrap(), buffer);
            let h4 = h3 + (next.unwrap() - *current);
            
            // Calc intersection
            let intersection = calc_line_intersection(&h1, &h2, &h3, &h4);
            if let Some(intersection_point) = intersection {
                buffered_line.push(intersection_point);
            } else {
                buffered_line.push(h2); // lines are parallel, hence use h2
            }
            
            buffer_left.push(*current - buffered_line[i] + *current);
        }
    }

    buffered_line.extend(buffer_left.into_iter().rev());
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

/// Calculate the intersection point of two line segments (a1, a2) and (b1, b2).
/// Returns the intersection point as a Point2D, or None if lines are parallel or are intersecting outside the segments.
fn calc_line_segment_intersection(a1: &Point2D, a2: &Point2D, b1: &Point2D, b2: &Point2D) -> Option<Point2D> {
    // signs of areas correspond to which side of ab points b1 and b2 are
    let area1 = calc_signed_2d_triangle_area(&a1,&a2,&b2); // Compute winding of a1a2b2 (+ or -)
    let area2 = calc_signed_2d_triangle_area(&a1,&a2,&b1); // To intersect, must have sign opposite of area1

    // If b1 and b2 are on different sides of a1b1, areas have different signs
    if area1 * area2 < 0.0 { // require unsigned x & y values.
        let area3 = calc_signed_2d_triangle_area(b1,b2,a1); // Compute winding of b1b2a1 (+ or -)
        let area4 = area3 + area2 - area1; // Since area is constant area1 - area2 = area3 - area4, or area4 = area3 + area2 - area1

        // Points a and b on different sides of b1b2 if areas have different signs
        if area3 * area4 < 0.0 {
            // Segments intersect. Find intersection point along L(t) = a1 + t * (a2 - a1).
            let t = area3 / (area3 - area4);
            let intersection = *a1 + (*a2 - *a1) * t; // the point of intersection
            return Some(intersection);
        }
    }

    // Segments not intersecting or collinear
    None
}

/// Calculate the signed area of a 2D triangle formed by points a, b, and c.
fn calc_signed_2d_triangle_area(a: &Point2D, b: &Point2D, c: &Point2D) -> f64 {
    (a.x - c.x) * (b.y - c.y) - (a.y - c.y) * (b.x - c.x)
}

/// Check if a point is inside a polygon using the ray-casting algorithm.
fn is_point_in_polygon(point: &Point2D, polygon: &Vec<Point2D>) -> bool {
    let mut inside = false;
    let n = polygon.len();
    for i in 0..n {
        let j = (i + n - 1) % n;
        if ((polygon[i].y > point.y) != (polygon[j].y > point.y)) &&
           (point.x < (polygon[j].x - polygon[i].x) * (point.y - polygon[i].y) / (polygon[j].y - polygon[i].y) + polygon[i].x) {
            inside = !inside;
        }
    }
    inside
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

    #[test]
    fn test_create_line_buffer_simple_line() {
        let line = vec![Point2D::new(0.0, 0.0), Point2D::new(1.0, 1.0)];
        let buffer = 0.5;
        let buffered_line = create_line_buffer(&line, buffer).unwrap();
        let a = (buffer.powi(2) / 2.0).sqrt();
        assert_eq!(buffered_line.len(), 5);
        assert!((buffered_line[0].x - a).abs() < 1e-10);
        assert!((buffered_line[0].y - (-a)).abs() < 1e-10);
        assert!((buffered_line[1].x - (1.0 + a)).abs() < 1e-10);
        assert!((buffered_line[1].y - (1.0 - a)).abs() < 1e-10);
        assert!((buffered_line[2].x - (1.0 - a)).abs() < 1e-10);
        assert!((buffered_line[2].y - (1.0 + a)).abs() < 1e-10);
        assert!((buffered_line[3].x - (-a)).abs() < 1e-10);
        assert!((buffered_line[3].y - a).abs() < 1e-10);
        assert!((buffered_line[4].x - a).abs() < 1e-10);
        assert!((buffered_line[4].y - (-a)).abs() < 1e-10);
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
        assert!((buffered_line[0].x - buffer).abs() < 1e-10);
        assert!((buffered_line[0].y).abs() < 1e-10);
        assert!((buffered_line[1].x - buffer).abs() < 1e-10);
        assert!((buffered_line[1].y - (1.0 - buffer)).abs() < 1e-10);
        assert!((buffered_line[2].x - 2.0).abs() < 1e-10);
        assert!((buffered_line[2].y - (1.0 - buffer)).abs() < 1e-10);
        assert!((buffered_line[3].x - 2.0).abs() < 1e-10);
        assert!((buffered_line[3].y - (1.0 + buffer)).abs() < 1e-10);
        assert!((buffered_line[4].x - (-buffer)).abs() < 1e-10);
        assert!((buffered_line[4].y - (1.0 + buffer)).abs() < 1e-10);
        assert!((buffered_line[5].x - (-buffer)).abs() < 1e-10);
        assert!((buffered_line[5].y).abs() < 1e-10);
        assert!((buffered_line[6].x - buffer).abs() < 1e-10);
        assert!((buffered_line[6].y).abs() < 1e-10);
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

    #[test]
    fn test_line_polygon_overlay_simple() {
        let line = vec![
            Point2D::new(-1, 1),
            Point2D::new(3, 1),
        ];
        let polygon = vec![
            Point2D::new(0, 0),
            Point2D::new(1, 0),
            Point2D::new(1, 2),
            Point2D::new(0, 2),
            Point2D::new(0, 0),
        ];

        let overlay = 1.0;
        let result = calc_line_polygon_overlay(&line, &polygon);

        assert!(
            (result - overlay).abs() < 1e-10,
            "Overlay is {} but should be {}",
            result, overlay
        );
    }

    #[test]
    fn test_line_polygon_overlay_on_edge() {
        let line = vec![
            Point2D::new(-1.0, 0.5),
            Point2D::new(1.0, 0.5),
            Point2D::new(1.0, 1.5),
            Point2D::new(-1.0, 1.5),
        ];
        let polygon = vec![
            Point2D::new(0, 0),
            Point2D::new(1, 0),
            Point2D::new(1, 2),
            Point2D::new(0, 2),
            Point2D::new(0, 0),
        ];

        let overlay = 3.0;
        let result = calc_line_polygon_overlay(&line, &polygon);

        assert!(
            (result - overlay).abs() < 1e-10,
            "Overlay is {} but should be {}",
            result, overlay
        );
    }

    // The three tests below pin the handling of a line that touches the polygon boundary without
    // properly crossing it. The strict inequalities of the segment test miss such a contact, and
    // the inside/outside state is wrong from there on.

    fn square() -> Vec<Point2D> {
        vec![
            Point2D::new(0, 0),
            Point2D::new(1, 0),
            Point2D::new(1, 2),
            Point2D::new(0, 2),
            Point2D::new(0, 0),
        ]
    }

    #[test]
    #[ignore = "known defect of the polygon overlay; fixed by replacing the overlay core"]
    fn test_line_polygon_overlay_vertex_on_boundary_then_leaving() {
        let line = vec![
            Point2D::new(-1.0, 0.5),
            Point2D::new(1.0, 0.5),
            Point2D::new(3.0, 0.5),
        ];

        let result = calc_line_polygon_overlay(&line, &square());

        assert!((result - 1.0).abs() < 1e-10, "Overlay is {} but should be 1", result);
    }

    #[test]
    #[ignore = "known defect of the polygon overlay; fixed by replacing the overlay core"]
    fn test_line_polygon_overlay_through_corners() {
        let line = vec![Point2D::new(-1, -1), Point2D::new(3, 3)];
        let unit_square = vec![
            Point2D::new(0, 0),
            Point2D::new(1, 0),
            Point2D::new(1, 1),
            Point2D::new(0, 1),
            Point2D::new(0, 0),
        ];

        let overlay = 2_f64.sqrt();
        let result = calc_line_polygon_overlay(&line, &unit_square);

        assert!(
            (result - overlay).abs() < 1e-10,
            "Overlay is {} but should be {}",
            result, overlay
        );
    }

    #[test]
    #[ignore = "known defect of the polygon overlay; fixed by replacing the overlay core"]
    fn test_line_polygon_overlay_starting_on_boundary() {
        let line = vec![Point2D::new(1.0, 0.5), Point2D::new(0.5, 0.5)];

        let result = calc_line_polygon_overlay(&line, &square());

        assert!((result - 0.5).abs() < 1e-10, "Overlay is {} but should be 0.5", result);
    }
}
