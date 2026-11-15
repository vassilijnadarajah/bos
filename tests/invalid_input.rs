// SPDX-License-Identifier: GPL-3.0-only
// (C) 2025 Vassilij Nadarajah
// packages@vassilijnadarajah.de

//! Input for which a metric does not exist is an error for both metrics, never a number and never
//! a panic.

use bos_geo::{
    EndCap, LineRole, MetricError, calc_completeness, calc_completeness_with_end_cap,
    calc_correctness, calc_correctness_with_end_cap,
};

type Line = Vec<[f64; 2]>;

fn valid_line() -> Line {
    vec![[0.0, 0.0], [1.0, 0.5], [2.0, 0.0]]
}

/// Asserts that every entry point reports `expected` for this input.
#[track_caller]
fn assert_error(prediction: &Line, ground_truth: &Line, buffer: f64, expected: MetricError) {
    assert_eq!(
        calc_completeness(prediction, ground_truth, buffer),
        Err(expected)
    );
    assert_eq!(
        calc_correctness(prediction, ground_truth, buffer),
        Err(expected)
    );
    for end_cap in [EndCap::Round, EndCap::Flat] {
        assert_eq!(
            calc_completeness_with_end_cap(prediction, ground_truth, buffer, end_cap),
            Err(expected)
        );
        assert_eq!(
            calc_correctness_with_end_cap(prediction, ground_truth, buffer, end_cap),
            Err(expected)
        );
    }
}

#[test]
fn empty_line() {
    let empty: Line = vec![];
    assert_error(
        &valid_line(),
        &empty,
        0.1,
        MetricError::TooFewPoints {
            line: LineRole::GroundTruth,
            points: 0,
        },
    );
    assert_error(
        &empty,
        &valid_line(),
        0.1,
        MetricError::TooFewPoints {
            line: LineRole::Prediction,
            points: 0,
        },
    );
}

#[test]
fn single_point() {
    let point: Line = vec![[1.0, 1.0]];
    assert_error(
        &valid_line(),
        &point,
        0.1,
        MetricError::TooFewPoints {
            line: LineRole::GroundTruth,
            points: 1,
        },
    );
    assert_error(
        &point,
        &valid_line(),
        0.1,
        MetricError::TooFewPoints {
            line: LineRole::Prediction,
            points: 1,
        },
    );
}

#[test]
fn all_points_identical() {
    let repeated: Line = vec![[1.0, 1.0]; 3];
    assert_error(
        &valid_line(),
        &repeated,
        0.1,
        MetricError::ZeroLength {
            line: LineRole::GroundTruth,
        },
    );
    assert_error(
        &repeated,
        &valid_line(),
        0.1,
        MetricError::ZeroLength {
            line: LineRole::Prediction,
        },
    );
}

#[test]
fn non_finite_coordinate() {
    for bad in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        let line: Line = vec![[0.0, 0.0], [1.0, bad], [2.0, 0.0]];
        assert_error(
            &valid_line(),
            &line,
            0.1,
            MetricError::NonFinite {
                line: LineRole::GroundTruth,
                index: 1,
            },
        );
        let line: Line = vec![[bad, 0.0], [1.0, 0.0]];
        assert_error(
            &line,
            &valid_line(),
            0.1,
            MetricError::NonFinite {
                line: LineRole::Prediction,
                index: 0,
            },
        );
    }
}

#[test]
fn buffer_not_positive_or_not_finite() {
    for buffer in [0.0, -0.0, -0.1, f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        assert_error(
            &valid_line(),
            &valid_line(),
            buffer,
            MetricError::InvalidBuffer,
        );
    }
}

/// The prediction is reported when both lines are unusable, and the buffer before either line.
#[test]
fn first_problem_in_argument_order_is_reported() {
    let empty: Line = vec![];
    let point: Line = vec![[1.0, 1.0]];
    assert_error(
        &empty,
        &point,
        0.1,
        MetricError::TooFewPoints {
            line: LineRole::Prediction,
            points: 0,
        },
    );
    assert_error(&empty, &point, 0.0, MetricError::InvalidBuffer);
}

#[test]
fn messages_name_the_argument() {
    let cases = [
        (
            MetricError::TooFewPoints {
                line: LineRole::Prediction,
                points: 1,
            },
            "prediction has 1 point(s), a line needs at least 2",
        ),
        (
            MetricError::ZeroLength {
                line: LineRole::GroundTruth,
            },
            "ground_truth has no length: all its points coincide",
        ),
        (
            MetricError::NonFinite {
                line: LineRole::GroundTruth,
                index: 7,
            },
            "ground_truth has a coordinate that is NaN or infinite at index 7",
        ),
        (
            MetricError::InvalidBuffer,
            "buffer must be a finite number greater than 0",
        ),
    ];
    for (error, message) in cases {
        assert_eq!(error.to_string(), message);
    }
}

/// A point listed twice in a row is dropped, not rejected: the result is that of the line
/// without the repetition.
#[test]
fn repeated_points_are_dropped() {
    let prediction: Line = vec![[0.0, 0.0], [1.0, 0.5], [2.0, 0.0]];
    let ground_truth: Line = vec![[0.0, 0.1], [1.2, 0.4], [2.5, 0.1]];
    let repeat = |line: &Line| -> Line { line.iter().flat_map(|&p| [p, p, p]).collect() };

    for end_cap in [EndCap::Round, EndCap::Flat] {
        let completeness =
            calc_completeness_with_end_cap(&prediction, &ground_truth, 0.15, end_cap);
        let correctness = calc_correctness_with_end_cap(&prediction, &ground_truth, 0.15, end_cap);
        assert!(completeness.is_ok_and(|value| value > 0.0 && value < 1.0));

        for (prediction, ground_truth) in [
            (repeat(&prediction), ground_truth.clone()),
            (prediction.clone(), repeat(&ground_truth)),
        ] {
            assert_eq!(
                calc_completeness_with_end_cap(&prediction, &ground_truth, 0.15, end_cap),
                completeness
            );
            assert_eq!(
                calc_correctness_with_end_cap(&prediction, &ground_truth, 0.15, end_cap),
                correctness
            );
        }
    }
}
