// SPDX-License-Identifier: GPL-3.0-only
// (C) 2025 Vassilij Nadarajah
// packages@vassilijnadarajah.de

//! The two metrics and the error they report.
//!
//! Both follow the same recipe and differ in which line is buffered: lay a buffer around one
//! line and measure how much of the other line's length lies inside it.

use std::fmt;

pub use crate::geometry::EndCap;
use crate::geometry::covered_fraction;
use crate::point2d::{AsPoint2D, Point2D};

/// One of the two lines that a metric takes, named like the argument it is passed as.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LineRole {
    /// The line being judged.
    Prediction,
    /// The line taken as correct.
    GroundTruth,
}

impl fmt::Display for LineRole {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            LineRole::Prediction => "prediction",
            LineRole::GroundTruth => "ground_truth",
        })
    }
}

/// Why a metric could not be computed.
///
/// A metric is a ratio of lengths. Input for which that ratio does not exist is reported as an
/// error, never as a number: a score of 0 for unusable input could not be told from a real 0.
///
/// The message names the argument concerned:
///
/// ```
/// use bos_geo::{LineRole, MetricError, calc_completeness};
///
/// let prediction = [[0.0, 0.0], [1.0, 0.0]];
/// let one_point = [[0.5, 0.0]];
/// let error = calc_completeness(&prediction, &one_point, 0.1).unwrap_err();
/// assert_eq!(error, MetricError::TooFewPoints { line: LineRole::GroundTruth, points: 1 });
/// assert_eq!(error.to_string(), "ground_truth has 1 point(s), a line needs at least 2");
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum MetricError {
    /// The line has fewer than two points.
    TooFewPoints {
        /// The line concerned.
        line: LineRole,
        /// The number of points it has.
        points: usize,
    },
    /// All points of the line coincide, so it has no length.
    ZeroLength {
        /// The line concerned.
        line: LineRole,
    },
    /// A coordinate of the line is NaN or infinite.
    NonFinite {
        /// The line concerned.
        line: LineRole,
        /// Index of the first point with such a coordinate.
        index: usize,
    },
    /// The buffer is not a finite number greater than zero.
    InvalidBuffer,
}

impl fmt::Display for MetricError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MetricError::TooFewPoints { line, points } => {
                write!(f, "{line} has {points} point(s), a line needs at least 2")
            }
            MetricError::ZeroLength { line } => {
                write!(f, "{line} has no length: all its points coincide")
            }
            MetricError::NonFinite { line, index } => {
                write!(
                    f,
                    "{line} has a coordinate that is NaN or infinite at index {index}"
                )
            }
            MetricError::InvalidBuffer => {
                f.write_str("buffer must be a finite number greater than 0")
            }
        }
    }
}

impl std::error::Error for MetricError {}

/// Completeness (recall): the share of the ground truth that was found.
///
/// ```text
/// Length(ground truth within the buffer of the prediction) / Length(ground truth)
/// ```
///
/// # Arguments
///
/// - `prediction`: the line being judged, as its points in order.
/// - `ground_truth`: the line taken as correct, as its points in order.
/// - `buffer`: the tolerance, as a distance in the unit of the coordinates. A point of the ground
///   truth is found if it is at most this far from the prediction. This is the half-width of the
///   buffer, which is everything within that distance and so has round ends; see
///   [`calc_completeness_with_end_cap`] for flat ends.
///
/// Coordinates are planar: both lines must be in the same projected coordinate system.
///
/// # Errors
///
/// A [`MetricError`] if either line has fewer than two distinct points or a coordinate that is
/// NaN or infinite, or if `buffer` is not a finite number greater than zero. A point repeated in
/// a row is ignored.
///
/// # Examples
///
/// ```
/// use bos_geo::calc_completeness;
///
/// // The prediction stops half way along the ground truth, and its buffer 0.1 later.
/// let prediction = [[0.0, 0.0], [5.0, 0.0]];
/// let ground_truth = [[0.0, 0.0], [10.0, 0.0]];
/// let completeness = calc_completeness(&prediction, &ground_truth, 0.1)?;
/// assert!((completeness - 0.51).abs() < 1e-12);
/// # Ok::<(), bos_geo::MetricError>(())
/// ```
pub fn calc_completeness<T: AsPoint2D>(
    prediction: &[T],
    ground_truth: &[T],
    buffer: f64,
) -> Result<f64, MetricError> {
    calc_completeness_with_end_cap(prediction, ground_truth, buffer, EndCap::default())
}

/// [`calc_completeness`] with a choice of the buffer's shape at the two ends of the prediction.
///
/// With [`EndCap::Flat`], ground truth that continues past an end of the prediction is not found
/// there. [`EndCap::Round`] is what [`calc_completeness`] uses.
///
/// # Errors
///
/// As for [`calc_completeness`].
///
/// # Examples
///
/// ```
/// use bos_geo::{EndCap, calc_completeness_with_end_cap};
///
/// let prediction = [[0.0, 0.0], [5.0, 0.0]];
/// let ground_truth = [[0.0, 0.0], [10.0, 0.0]];
/// let completeness =
///     calc_completeness_with_end_cap(&prediction, &ground_truth, 0.1, EndCap::Flat)?;
/// assert!((completeness - 0.5).abs() < 1e-12);
/// # Ok::<(), bos_geo::MetricError>(())
/// ```
pub fn calc_completeness_with_end_cap<T: AsPoint2D>(
    prediction: &[T],
    ground_truth: &[T],
    buffer: f64,
    end_cap: EndCap,
) -> Result<f64, MetricError> {
    let (prediction, ground_truth) = validate(prediction, ground_truth, buffer)?;
    // Length(ground truth inside buffer(prediction)) / Length(ground truth)
    covered_fraction(&ground_truth, &prediction, buffer, end_cap).ok_or(MetricError::ZeroLength {
        line: LineRole::GroundTruth,
    })
}

/// Correctness (precision): the share of the prediction that is right.
///
/// ```text
/// Length(prediction within the buffer of the ground truth) / Length(prediction)
/// ```
///
/// One minus this value is what Tveite and Langaas call miscodings.
///
/// # Arguments
///
/// - `prediction`: the line being judged, as its points in order.
/// - `ground_truth`: the line taken as correct, as its points in order.
/// - `buffer`: the tolerance, as a distance in the unit of the coordinates. A point of the
///   prediction is right if it is at most this far from the ground truth. This is the half-width
///   of the buffer, which is everything within that distance and so has round ends; see
///   [`calc_correctness_with_end_cap`] for flat ends.
///
/// Coordinates are planar: both lines must be in the same projected coordinate system.
///
/// # Errors
///
/// A [`MetricError`] if either line has fewer than two distinct points or a coordinate that is
/// NaN or infinite, or if `buffer` is not a finite number greater than zero. A point repeated in
/// a row is ignored.
///
/// # Examples
///
/// ```
/// use bos_geo::calc_correctness;
///
/// // The prediction follows the ground truth for 4 units and then leaves it at a right angle.
/// let prediction = [[0.0, 0.0], [4.0, 0.0], [4.0, 4.0]];
/// let ground_truth = [[0.0, 0.0], [10.0, 0.0]];
/// let correctness = calc_correctness(&prediction, &ground_truth, 0.1)?;
/// assert!((correctness - 4.1 / 8.0).abs() < 1e-12);
/// # Ok::<(), bos_geo::MetricError>(())
/// ```
pub fn calc_correctness<T: AsPoint2D>(
    prediction: &[T],
    ground_truth: &[T],
    buffer: f64,
) -> Result<f64, MetricError> {
    calc_correctness_with_end_cap(prediction, ground_truth, buffer, EndCap::default())
}

/// [`calc_correctness`] with a choice of the buffer's shape at the two ends of the ground truth.
///
/// With [`EndCap::Flat`], a prediction that continues past an end of the ground truth is not
/// right there. [`EndCap::Round`] is what [`calc_correctness`] uses.
///
/// # Errors
///
/// As for [`calc_correctness`].
///
/// # Examples
///
/// ```
/// use bos_geo::{EndCap, calc_correctness_with_end_cap};
///
/// // The prediction overshoots the end of the ground truth by 1.
/// let prediction = [[0.0, 0.0], [5.0, 0.0]];
/// let ground_truth = [[0.0, 0.0], [4.0, 0.0]];
/// let round = calc_correctness_with_end_cap(&prediction, &ground_truth, 0.1, EndCap::Round)?;
/// let flat = calc_correctness_with_end_cap(&prediction, &ground_truth, 0.1, EndCap::Flat)?;
/// assert!((round - 0.82).abs() < 1e-12);
/// assert!((flat - 0.8).abs() < 1e-12);
/// # Ok::<(), bos_geo::MetricError>(())
/// ```
pub fn calc_correctness_with_end_cap<T: AsPoint2D>(
    prediction: &[T],
    ground_truth: &[T],
    buffer: f64,
    end_cap: EndCap,
) -> Result<f64, MetricError> {
    let (prediction, ground_truth) = validate(prediction, ground_truth, buffer)?;
    // Length(prediction inside buffer(ground truth)) / Length(prediction)
    covered_fraction(&prediction, &ground_truth, buffer, end_cap).ok_or(MetricError::ZeroLength {
        line: LineRole::Prediction,
    })
}

/// Checks the input of a metric and returns the two lines ready for the computation.
///
/// Both metrics need both lines to be lines, whichever of them is buffered: at least two distinct
/// points, all coordinates finite. The buffer must be finite and greater than zero.
fn validate<T: AsPoint2D>(
    prediction: &[T],
    ground_truth: &[T],
    buffer: f64,
) -> Result<(Vec<Point2D>, Vec<Point2D>), MetricError> {
    if !(buffer.is_finite() && buffer > 0.0) {
        return Err(MetricError::InvalidBuffer);
    }
    let prediction = clean_line(prediction, LineRole::Prediction)?;
    let ground_truth = clean_line(ground_truth, LineRole::GroundTruth)?;
    Ok((prediction, ground_truth))
}

/// The points of `line` without consecutive repetitions, or the reason why it is not a line.
fn clean_line<T: AsPoint2D>(line: &[T], role: LineRole) -> Result<Vec<Point2D>, MetricError> {
    if line.len() < 2 {
        return Err(MetricError::TooFewPoints {
            line: role,
            points: line.len(),
        });
    }
    let mut points: Vec<Point2D> = Vec::with_capacity(line.len());
    for (index, point) in line.iter().enumerate() {
        let point = point.to_point2d();
        if !(point.x.is_finite() && point.y.is_finite()) {
            return Err(MetricError::NonFinite { line: role, index });
        }
        // A point listed twice in a row is common in exported polylines and says nothing.
        if points.last() != Some(&point) {
            points.push(point);
        }
    }
    if points.len() < 2 {
        return Err(MetricError::ZeroLength { line: role });
    }
    Ok(points)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::point2d::Point2D;

    const END_CAPS: [EndCap; 2] = [EndCap::Round, EndCap::Flat];

    fn simple_lines() -> (Vec<Point2D>, Vec<Point2D>) {
        let prediction = vec![Point2D::new(0, 0), Point2D::new(2, 2)];
        let ground_truth = vec![Point2D::new(1, 1), Point2D::new(3, 3)];
        (prediction, ground_truth)
    }

    fn complex_lines() -> (Vec<Point2D>, Vec<Point2D>) {
        let prediction = vec![
            Point2D::new(-1, -2),
            Point2D::new(-1, 0),
            Point2D::new(0, 1),
            Point2D::new(2, 1),
        ];
        let ground_truth = vec![
            Point2D::new(-1, -2),
            Point2D::new(-1, -1),
            Point2D::new(1, 1),
            Point2D::new(2, 1),
        ];
        (prediction, ground_truth)
    }

    #[test]
    fn test_completeness_simple() {
        let (prediction, ground_truth) = simple_lines();
        let buffer = 0.1;

        // Half of the ground truth runs along the prediction. The round end of the buffer
        // covers another `buffer` of it past the end of the prediction.
        let result = calc_completeness(&prediction, &ground_truth, buffer).unwrap();
        let completeness = 0.5 + buffer / (2.0 * 2_f64.sqrt());
        assert!(
            (result - completeness).abs() < 1e-10,
            "Completeness is {} but should be {}",
            result,
            completeness
        );
    }

    #[test]
    fn test_completeness_simple_flat_ends() {
        let (prediction, ground_truth) = simple_lines();
        let buffer = 0.1;

        let result =
            calc_completeness_with_end_cap(&prediction, &ground_truth, buffer, EndCap::Flat)
                .unwrap();
        let completeness = 0.5;
        assert!(
            (result - completeness).abs() < 1e-10,
            "Completeness is {} but should be {}",
            result,
            completeness
        );
    }

    #[test]
    fn test_completeness_complex() {
        // Both lines share their end points, so the shape of the buffer ends does not matter.
        let (prediction, ground_truth) = complex_lines();
        let buffer = 0.2;
        let completeness = (2.0 * 2_f64.sqrt() * buffer + 2.0) / (2.0 * 2_f64.sqrt() + 2.0);

        for end_cap in END_CAPS {
            let result =
                calc_completeness_with_end_cap(&prediction, &ground_truth, buffer, end_cap)
                    .unwrap();
            assert!(
                (result - completeness).abs() < 1e-10,
                "Completeness is {} but should be {} ({:?})",
                result,
                completeness,
                end_cap
            );
        }
    }

    #[test]
    fn test_correctness_simple() {
        let (prediction, ground_truth) = simple_lines();
        let buffer = 0.1;

        let result = calc_correctness(&prediction, &ground_truth, buffer).unwrap();
        let correctness = 0.5 + buffer / (2.0 * 2_f64.sqrt());
        assert!(
            (result - correctness).abs() < 1e-10,
            "Correctness is {} but should be {}",
            result,
            correctness
        );
    }

    #[test]
    fn test_correctness_simple_flat_ends() {
        let (prediction, ground_truth) = simple_lines();
        let buffer = 0.1;

        let result =
            calc_correctness_with_end_cap(&prediction, &ground_truth, buffer, EndCap::Flat)
                .unwrap();
        let correctness = 0.5;
        assert!(
            (result - correctness).abs() < 1e-10,
            "Correctness is {} but should be {}",
            result,
            correctness
        );
    }

    #[test]
    fn test_correctness_complex() {
        let (prediction, ground_truth) = complex_lines();
        let buffer = 0.2;
        let correctness = (2.0 * 2_f64.sqrt() * buffer + 2.0) / (2_f64.sqrt() + 4.0);

        for end_cap in END_CAPS {
            let result =
                calc_correctness_with_end_cap(&prediction, &ground_truth, buffer, end_cap).unwrap();
            assert!(
                (result - correctness).abs() < 1e-10,
                "Correctness is {} but should be {} ({:?})",
                result,
                correctness,
                end_cap
            );
        }
    }
}
