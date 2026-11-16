// SPDX-License-Identifier: GPL-3.0-only
// (C) 2025 Vassilij Nadarajah
// packages@vassilijnadarajah.de

//! Buffer overlay statistics for 2D polylines.
//!
//! Scores a predicted line against a ground truth line with the buffer method of Tveite and
//! Langaas (1999): a part of one line counts as matched if it lies within a tolerance, the
//! buffer, of the other line.
//!
//! | Function | Value | Also called |
//! |---|---|---|
//! | [`calc_completeness`] | length of the ground truth within the buffer of the prediction, divided by the length of the ground truth | recall |
//! | [`calc_correctness`] | length of the prediction within the buffer of the ground truth, divided by the length of the prediction | precision |
//!
//! # Conventions
//!
//! - Arguments are always `(prediction, ground_truth, buffer)`.
//! - A line is a slice of points in order, of any type that implements [`AsPoint2D`]: [`Point2D`],
//!   `[x, y]` or `(x, y)`. One call takes one prediction and one ground truth line.
//! - Coordinates are planar. Both lines and the buffer must be in the same unit and the same
//!   projected coordinate system; degrees of longitude and latitude give meaningless results.
//! - `buffer` is a distance, the half-width of the buffer: a point is in the buffer of a line if
//!   it is at most `buffer` away from it, so the buffer is `2 · buffer` wide. It reaches that
//!   distance past the ends of the line as well; [`EndCap::Flat`] cuts it off there.
//! - A result is a share between 0 and 1. Input without a result is a [`MetricError`].
//!
//! # Example
//!
//! The prediction runs 0.3 beside the first part of a ground truth that is twice as long.
//!
//! ```
//! use bos_geo::{EndCap, calc_completeness, calc_correctness, calc_correctness_with_end_cap};
//!
//! let prediction = [[0.0, 0.0], [4.0, 0.0]];
//! let ground_truth = [[1.0, 0.3], [9.0, 0.3]];
//!
//! // The prediction is 0.3 from the ground truth between x = 1 and x = 4. Its round end reaches
//! // another sqrt(0.5² - 0.3²) = 0.4 along the ground truth: 3.4 of 8 are found.
//! let completeness = calc_completeness(&prediction, &ground_truth, 0.5)?;
//! assert!((completeness - 0.425).abs() < 1e-12);
//!
//! // By the same reasoning 3.4 of the 4 units of prediction are right.
//! let correctness = calc_correctness(&prediction, &ground_truth, 0.5)?;
//! assert!((correctness - 0.85).abs() < 1e-12);
//!
//! // With flat ends only the 3 units side by side count.
//! let flat = calc_correctness_with_end_cap(&prediction, &ground_truth, 0.5, EndCap::Flat)?;
//! assert!((flat - 0.75).abs() < 1e-12);
//! # Ok::<(), bos_geo::MetricError>(())
//! ```
//!
//! # What the method does not see
//!
//! Only distance counts. A prediction that crosses the ground truth at a right angle is credited
//! with the stretch inside the buffer, a prediction that wiggles inside the buffer scores like a
//! straight one, and a ground truth found twice scores like one found once.

#![warn(missing_docs)]

mod geometry;
pub mod metrics;
pub mod point2d;

#[cfg(feature = "python")]
mod python;

pub use geometry::EndCap;
pub use metrics::{
    LineRole, MetricError, calc_completeness, calc_completeness_with_end_cap, calc_correctness,
    calc_correctness_with_end_cap,
};
pub use point2d::{AsPoint2D, Point2D};

/// Runs the Rust example of the README as a documentation test.
#[cfg(doctest)]
#[doc = include_str!("../README.md")]
struct ReadmeExamples;
