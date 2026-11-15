// SPDX-License-Identifier: GPL-3.0-only
// (C) 2025 Vassilij Nadarajah
// packages@vassilijnadarajah.de

//! How much of one polyline lies within a given distance of another.
//!
//! The buffer of a polyline is the set of points within distance `r` of it. It is the union of
//! simple convex pieces: a rectangle of half-width `r` around every segment and a disc of radius
//! `r` around every vertex. A straight segment meets a convex piece in at most one interval of its
//! parameter `t`, and for a rectangle and a disc that interval has a closed form. The part of a
//! segment inside the buffer is the union of these intervals.
//!
//! With flat ends ([`EndCap::Flat`]) the half discs beyond the two ends of the line are left out.
//! What remains of the disc around an interior vertex is the sector on the outside of the bend,
//! between the rectangles of the two segments that meet there. It is convex as well.
//!
//! No buffer polygon is built and no point is ever classified as inside or outside, so there is
//! nothing that could go wrong where a line touches the buffer boundary, where the buffer overlaps
//! itself or where two segments are almost parallel.

use crate::point2d::Point2D;

/// Shape of the buffer at the two ends of the buffered line.
///
/// Along the line and at its interior vertices the buffer is always everything within the buffer
/// distance, so joins between segments are round.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum EndCap {
    /// The buffer is everything within the buffer distance of the line. It reaches past the ends
    /// of the line by that distance, in a half disc. This is the buffer of a GIS buffer operation.
    #[default]
    Round,
    /// The buffer is cut off square at the first and the last vertex: the half discs that a round
    /// buffer has beyond the two ends of the line are left out.
    Flat,
}

/// A closed interval `[start, end]` of the parameter of a segment, with `0 <= start <= end <= 1`.
type Interval = (f64, f64);

/// Two covered intervals that are closer than this, as a fraction of the segment, are joined.
/// Neighbouring pieces of the buffer overlap, but rounding can open a gap of a few ulps between
/// the intervals they produce.
const MERGE_TOLERANCE: f64 = 1e-12;

/// Share of the length of `line` that lies in the buffer of `buffered`, in `[0, 1]`.
///
/// `radius` is the buffer distance. Returns `None` if `line` has no length.
pub(crate) fn covered_fraction(
    line: &[Point2D],
    buffered: &[Point2D],
    radius: f64,
    end_cap: EndCap,
) -> Option<f64> {
    // Work relative to a vertex of the data. Projected coordinates are often in the millions (UTM
    // northings), where a product of two of them has no digits left for centimetres. The formulas
    // below use differences of coordinates throughout; this makes the numbers they start from
    // small as well.
    let origin = *buffered.first().or(line.first())?;
    let buffer = Buffer::new(buffered, origin, radius, end_cap);

    let mut total = 0.0;
    let mut covered = 0.0;
    let mut intervals = Vec::new();
    for pair in line.windows(2) {
        let segment = Segment::new(pair[0] - origin, pair[1] - origin);
        // Too short to have a direction: a point, which adds no length.
        if segment.length_squared == 0.0 {
            continue;
        }
        let length = segment.direction.abs();

        intervals.clear();
        buffer.intervals_on(&segment, &mut intervals);

        total += length;
        covered += length * union_length(&mut intervals).min(1.0);
    }

    (total > 0.0).then(|| covered / total)
}

/// A segment of the measured line: the points `start + t · direction` for `t` in `[0, 1]`.
struct Segment {
    start: Point2D,
    direction: Point2D,
    length_squared: f64,
}

impl Segment {
    fn new(start: Point2D, end: Point2D) -> Self {
        let direction = end - start;
        Segment {
            start,
            direction,
            length_squared: direction.dot(direction),
        }
    }
}

/// The buffer of a polyline, as the pieces it is the union of.
struct Buffer {
    radius: f64,
    end_cap: EndCap,
    pieces: Vec<BufferPiece>,
}

/// One segment of the buffered line. It stands for the rectangle around the segment and for the
/// join at its start vertex; the last piece also for the cap at its end vertex.
struct BufferPiece {
    start: Point2D,
    end: Point2D,
    direction: Point2D,
    length_squared: f64,
    /// Bound on `|(p - start) × direction|` for a point `p` of the rectangle: `radius · length`.
    cross_limit: f64,
    /// Direction of the piece that ends where this one starts; `None` for the first piece.
    incoming: Option<Point2D>,
    is_last: bool,
}

impl Buffer {
    fn new(line: &[Point2D], origin: Point2D, radius: f64, end_cap: EndCap) -> Self {
        let mut pieces: Vec<BufferPiece> = Vec::with_capacity(line.len().saturating_sub(1));
        let mut vertices = line.iter().map(|&vertex| vertex - origin);
        if let Some(mut start) = vertices.next() {
            for end in vertices {
                let direction = end - start;
                let length_squared = direction.dot(direction);
                // A repeated vertex. It has no direction and adds nothing to the buffer.
                if length_squared == 0.0 {
                    continue;
                }
                pieces.push(BufferPiece {
                    start,
                    end,
                    direction,
                    length_squared,
                    cross_limit: radius * direction.abs(),
                    incoming: pieces.last().map(|previous| previous.direction),
                    is_last: false,
                });
                start = end;
            }
        }
        if let Some(last) = pieces.last_mut() {
            last.is_last = true;
        }
        Buffer {
            radius,
            end_cap,
            pieces,
        }
    }

    /// Appends the intervals of `segment` that lie in the pieces of the buffer.
    fn intervals_on(&self, segment: &Segment, intervals: &mut Vec<Interval>) {
        for piece in &self.pieces {
            intervals.extend(rectangle_interval(segment, piece));
            match self.end_cap {
                // Everything within the radius: a full disc around every vertex.
                EndCap::Round => {
                    intervals.extend(disc_interval(segment, piece.start, self.radius));
                    if piece.is_last {
                        intervals.extend(disc_interval(segment, piece.end, self.radius));
                    }
                }
                // Nothing at the two ends, and at an interior vertex only what lies between
                // the rectangles of the two pieces.
                EndCap::Flat => {
                    if let Some(incoming) = piece.incoming {
                        intervals.extend(join_interval(
                            segment,
                            piece.start,
                            self.radius,
                            incoming,
                            piece.direction,
                        ));
                    }
                }
            }
        }
    }
}

/// The part of `segment` inside the disc of the given `radius` around `centre`.
fn disc_interval(segment: &Segment, centre: Point2D, radius: f64) -> Option<Interval> {
    let u = segment.direction;
    let w = segment.start - centre;
    // |w + t·u|² <= r²  is  (u·u)·t² + 2·(w·u)·t + (w·w - r²) <= 0. A quarter of its discriminant
    // is (w·u)² - (u·u)·(w·w - r²). By Lagrange's identity that equals (u·u)·r² - (w×u)², which
    // does not subtract two large, nearly equal numbers when the centre is far along the line.
    let cross = w.cross(u);
    let discriminant = segment.length_squared * radius * radius - cross * cross;
    if discriminant >= 0.0 {
        let root = discriminant.sqrt();
        let mid = -w.dot(u);
        clamp_to_segment(
            (mid - root) / segment.length_squared,
            (mid + root) / segment.length_squared,
        )
    } else {
        None
    }
}

/// The part of `segment` inside the round join at `vertex`, where a piece with direction `incoming`
/// ends and one with direction `outgoing` starts: the points of the disc around the vertex that
/// are past the end of the first piece and before the start of the second. That is the sector on
/// the outside of the bend. It is empty where the line goes straight on.
fn join_interval(
    segment: &Segment,
    vertex: Point2D,
    radius: f64,
    incoming: Point2D,
    outgoing: Point2D,
) -> Option<Interval> {
    let u = segment.direction;
    let w = segment.start - vertex;
    let in_disc = disc_interval(segment, vertex, radius)?;
    // Past the end of the incoming piece: (w + t·u)·incoming >= 0.
    let past_incoming = clip(
        in_disc,
        w.dot(incoming),
        u.dot(incoming),
        0.0,
        f64::INFINITY,
    )?;
    // Before the start of the outgoing piece: (w + t·u)·outgoing <= 0.
    clip(
        past_incoming,
        w.dot(outgoing),
        u.dot(outgoing),
        f64::NEG_INFINITY,
        0.0,
    )
}

/// The part of `segment` inside the rectangle of `piece`: the points whose foot on the line
/// through the piece lies on the piece, and which are at most the buffer distance from that line.
fn rectangle_interval(segment: &Segment, piece: &BufferPiece) -> Option<Interval> {
    let u = segment.direction;
    let v = piece.direction;
    let w = segment.start - piece.start;
    // Along the piece: 0 <= (w + t·u)·v <= v·v.
    let along = clip((0.0, 1.0), w.dot(v), u.dot(v), 0.0, piece.length_squared)?;
    // Across it: |(w + t·u)×v| <= r·|v|. In the plane the cross product is linear in t, so no
    // quadratic has to be solved and nearly parallel segments need no special care.
    clip(
        along,
        w.cross(v),
        u.cross(v),
        -piece.cross_limit,
        piece.cross_limit,
    )
}

/// Clips `interval` to the values of `t` with `lower <= constant + t · slope <= upper`.
fn clip(interval: Interval, constant: f64, slope: f64, lower: f64, upper: f64) -> Option<Interval> {
    if slope == 0.0 {
        return (lower <= constant && constant <= upper).then_some(interval);
    }
    let a = (lower - constant) / slope;
    let b = (upper - constant) / slope;
    let start = interval.0.max(a.min(b));
    let end = interval.1.min(a.max(b));
    (start <= end).then_some((start, end))
}

/// Restricts `[start, end]` to the segment's own parameter range `[0, 1]`.
fn clamp_to_segment(start: f64, end: f64) -> Option<Interval> {
    let start = start.max(0.0);
    let end = end.min(1.0);
    (start <= end).then_some((start, end))
}

/// Total length of the union of `intervals`. Sorts them in place.
fn union_length(intervals: &mut [Interval]) -> f64 {
    intervals.sort_unstable_by(|a, b| a.0.total_cmp(&b.0));
    let mut merged = intervals.iter().copied();
    let Some((mut start, mut end)) = merged.next() else {
        return 0.0;
    };
    let mut total = 0.0;
    for (next_start, next_end) in merged {
        if next_start <= end + MERGE_TOLERANCE {
            end = end.max(next_end);
        } else {
            total += end - start;
            (start, end) = (next_start, next_end);
        }
    }
    total + (end - start)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn segment(start: (f64, f64), end: (f64, f64)) -> Segment {
        Segment::new(Point2D::new(start.0, start.1), Point2D::new(end.0, end.1))
    }

    fn line(points: &[(f64, f64)]) -> Vec<Point2D> {
        points.iter().map(|&(x, y)| Point2D::new(x, y)).collect()
    }

    /// The single piece of the buffer of the segment `start`–`end`.
    fn piece(start: (f64, f64), end: (f64, f64), radius: f64) -> BufferPiece {
        let line = line(&[start, end]);
        let buffer = Buffer::new(&line, Point2D::new(0.0, 0.0), radius, EndCap::Round);
        buffer.pieces.into_iter().next().unwrap()
    }

    fn assert_interval(actual: Option<Interval>, expected: Interval) {
        let (start, end) = actual.expect("an interval");
        assert!(
            (start - expected.0).abs() < 1e-12 && (end - expected.1).abs() < 1e-12,
            "interval is {:?} but should be {:?}",
            (start, end),
            expected
        );
    }

    #[test]
    fn disc_interval_of_a_chord() {
        let centre = Point2D::new(0.0, 0.0);
        let through_centre = segment((-2.0, 0.0), (2.0, 0.0));
        assert_interval(disc_interval(&through_centre, centre, 1.0), (0.25, 0.75));

        // 0.6 above the centre the chord is 2 · 0.8 long.
        let off_centre = segment((-2.0, 0.6), (2.0, 0.6));
        assert_interval(disc_interval(&off_centre, centre, 1.0), (0.3, 0.7));
    }

    #[test]
    fn disc_interval_is_limited_to_the_segment() {
        let centre = Point2D::new(0.0, 0.0);
        let starting_inside = segment((0.0, 0.0), (2.0, 0.0));
        assert_interval(disc_interval(&starting_inside, centre, 1.0), (0.0, 0.5));

        let fully_inside = segment((-0.5, 0.0), (0.0, 0.5));
        assert_interval(disc_interval(&fully_inside, centre, 1.0), (0.0, 1.0));
    }

    #[test]
    fn disc_interval_of_a_tangent_is_a_single_point() {
        let tangent = segment((-2.0, 1.0), (2.0, 1.0));
        assert_interval(
            disc_interval(&tangent, Point2D::new(0.0, 0.0), 1.0),
            (0.5, 0.5),
        );
    }

    #[test]
    fn disc_interval_is_none_outside_the_disc() {
        let centre = Point2D::new(0.0, 0.0);
        let passing_by = segment((-2.0, 1.5), (2.0, 1.5));
        assert_eq!(disc_interval(&passing_by, centre, 1.0), None);

        // Its line meets the disc, the segment ends before it.
        let ending_before = segment((-5.0, 0.0), (-2.0, 0.0));
        assert_eq!(disc_interval(&ending_before, centre, 1.0), None);
    }

    #[test]
    fn rectangle_interval_of_a_crossing() {
        let piece = piece((0.0, 0.0), (4.0, 0.0), 0.5);
        let perpendicular = segment((2.0, -2.0), (2.0, 2.0));
        assert_interval(rectangle_interval(&perpendicular, &piece), (0.375, 0.625));

        // Enters through the side at (1.5, -0.5), leaves through the other side at (2.5, 0.5).
        let diagonal = segment((0.0, -2.0), (4.0, 2.0));
        assert_interval(rectangle_interval(&diagonal, &piece), (0.375, 0.625));
    }

    #[test]
    fn rectangle_interval_of_a_parallel_segment() {
        let piece = piece((0.0, 0.0), (4.0, 0.0), 0.5);
        let inside = segment((1.0, 0.25), (3.0, 0.25));
        assert_interval(rectangle_interval(&inside, &piece), (0.0, 1.0));

        let on_the_boundary = segment((1.0, 0.5), (3.0, 0.5));
        assert_interval(rectangle_interval(&on_the_boundary, &piece), (0.0, 1.0));

        let outside = segment((1.0, 1.0), (3.0, 1.0));
        assert_eq!(rectangle_interval(&outside, &piece), None);

        let longer = segment((-2.0, 0.25), (6.0, 0.25));
        assert_interval(rectangle_interval(&longer, &piece), (0.25, 0.75));
    }

    #[test]
    fn rectangle_ends_at_the_ends_of_its_piece() {
        let piece = piece((0.0, 0.0), (4.0, 0.0), 0.5);
        let beyond_the_end = segment((4.25, -1.0), (4.25, 1.0));
        assert_eq!(rectangle_interval(&beyond_the_end, &piece), None);

        let leaving_through_the_end = segment((3.0, 0.0), (5.0, 0.0));
        assert_interval(
            rectangle_interval(&leaving_through_the_end, &piece),
            (0.0, 0.5),
        );
    }

    #[test]
    fn join_interval_is_the_sector_outside_the_bend() {
        // A left turn at the origin, from heading east to heading north. The join is the quarter
        // disc to the south-east.
        let vertex = Point2D::new(0.0, 0.0);
        let (east, north) = (Point2D::new(3.0, 0.0), Point2D::new(0.0, 2.0));
        let join = |segment: &Segment| join_interval(segment, vertex, 1.0, east, north);

        // Along y = -0.6 the disc spans x in [-0.8, 0.8]; the sector keeps x >= 0.
        assert_interval(join(&segment((-2.0, -0.6), (2.0, -0.6))), (0.5, 0.7));
        // Along x = 0.6 the sector keeps y <= 0.
        assert_interval(join(&segment((0.6, -2.0), (0.6, 2.0))), (0.3, 0.5));
        // The inside of the bend belongs to the two rectangles, not to the join.
        assert_eq!(join(&segment((-0.9, 0.1), (-0.1, 0.9))), None);
    }

    #[test]
    fn join_interval_is_empty_where_the_line_goes_straight_on() {
        let vertex = Point2D::new(0.0, 0.0);
        let east = Point2D::new(1.0, 0.0);
        let crossing_beside_the_vertex = segment((0.25, -2.0), (0.25, 2.0));
        assert_eq!(
            join_interval(&crossing_beside_the_vertex, vertex, 1.0, east, east),
            None
        );
    }

    #[test]
    fn join_interval_of_a_reversal_is_a_half_disc() {
        let vertex = Point2D::new(0.0, 0.0);
        let (east, west) = (Point2D::new(1.0, 0.0), Point2D::new(-1.0, 0.0));
        let through = segment((-2.0, 0.0), (2.0, 0.0));
        assert_interval(
            join_interval(&through, vertex, 1.0, east, west),
            (0.5, 0.75),
        );
    }

    #[test]
    fn union_length_merges_overlapping_intervals() {
        assert_eq!(union_length(&mut []), 0.0);
        assert_eq!(union_length(&mut [(0.25, 0.5)]), 0.25);
        // Overlapping, nested and given out of order.
        assert_eq!(
            union_length(&mut [(0.5, 0.75), (0.0, 0.5), (0.125, 0.25)]),
            0.75
        );
        // Separate.
        assert_eq!(union_length(&mut [(0.75, 1.0), (0.0, 0.25)]), 0.5);
    }

    #[test]
    fn union_length_closes_hairline_gaps() {
        let gap = 1e-15;
        assert_eq!(union_length(&mut [(0.0, 0.5 - gap), (0.5, 1.0)]), 1.0);
    }

    #[test]
    fn covered_fraction_of_a_crossing_line() {
        let buffered = line(&[(0.0, 0.0), (4.0, 0.0)]);
        let crossing = line(&[(2.0, -2.0), (2.0, 2.0)]);
        let fraction = covered_fraction(&crossing, &buffered, 0.5, EndCap::Round).unwrap();
        assert!(
            (fraction - 0.25).abs() < 1e-12,
            "fraction is {fraction} but should be 0.25"
        );
    }

    #[test]
    fn end_cap_decides_what_is_covered_past_an_end() {
        let buffered = line(&[(0.0, 0.0), (4.0, 0.0)]);
        // Continues the buffered line by 2; the first 0.5 of it is within the buffer distance.
        let continuation = line(&[(4.0, 0.0), (6.0, 0.0)]);
        let round = covered_fraction(&continuation, &buffered, 0.5, EndCap::Round).unwrap();
        let flat = covered_fraction(&continuation, &buffered, 0.5, EndCap::Flat).unwrap();
        assert!((round - 0.25).abs() < 1e-12, "round: {round}");
        assert_eq!(flat, 0.0);
    }

    #[test]
    fn joins_are_round_with_either_end_cap() {
        // An L. Outside its corner the buffer is a quarter disc, which the diagonal x + y = 1.2
        // crosses in a chord of length 2 · sqrt(1 - 0.72); a mitre join would give 0.8 · sqrt(2).
        let buffered = line(&[(-5.0, 0.0), (0.0, 0.0), (0.0, -5.0)]);
        let diagonal = line(&[(-0.8, 2.0), (2.0, -0.8)]);
        let expected = 2.0 * 0.28_f64.sqrt() / (2.8 * 2_f64.sqrt());
        for end_cap in [EndCap::Round, EndCap::Flat] {
            let fraction = covered_fraction(&diagonal, &buffered, 1.0, end_cap).unwrap();
            assert!(
                (fraction - expected).abs() < 1e-12,
                "{end_cap:?}: {fraction}"
            );
        }
    }

    #[test]
    fn flat_ends_do_not_depend_on_where_the_vertices_are() {
        // The same straight line, once with a vertex close to its start. Flat ends cover nothing
        // of a line that crosses 0.25 before the start; round ends cover the chord of the disc.
        let plain = line(&[(0.0, 0.0), (4.0, 0.0)]);
        let with_vertex = line(&[(0.0, 0.0), (0.1, 0.0), (4.0, 0.0)]);
        let crossing = line(&[(-0.25, -1.0), (-0.25, 1.0)]);
        let chord = 0.1875_f64.sqrt();
        for buffered in [&plain, &with_vertex] {
            assert_eq!(
                covered_fraction(&crossing, buffered, 0.5, EndCap::Flat),
                Some(0.0)
            );
            let round = covered_fraction(&crossing, buffered, 0.5, EndCap::Round).unwrap();
            assert!((round - chord).abs() < 1e-12, "round: {round}");
        }
    }

    #[test]
    fn repeated_vertices_are_harmless() {
        let buffered = line(&[(0.0, 0.0), (2.0, 0.0), (2.0, 0.0), (4.0, 0.0)]);
        let measured = line(&[(0.0, 0.1), (0.0, 0.1), (4.0, 0.1)]);
        for end_cap in [EndCap::Round, EndCap::Flat] {
            assert_eq!(
                covered_fraction(&measured, &buffered, 0.2, end_cap),
                Some(1.0)
            );
        }
        // A line that is one point has no buffer.
        let point = line(&[(1.0, 0.1), (1.0, 0.1)]);
        assert_eq!(
            covered_fraction(&measured, &point, 0.2, EndCap::Round),
            Some(0.0)
        );
    }

    #[test]
    fn covered_fraction_is_none_for_a_line_without_length() {
        let buffered = line(&[(0.0, 0.0), (4.0, 0.0)]);
        assert_eq!(covered_fraction(&[], &buffered, 0.5, EndCap::Round), None);
        let point = line(&[(1.0, 0.0), (1.0, 0.0)]);
        assert_eq!(
            covered_fraction(&point, &buffered, 0.5, EndCap::Round),
            None
        );
    }
}
