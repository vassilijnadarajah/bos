// SPDX-License-Identifier: GPL-3.0-only
// (C) 2025 Vassilij Nadarajah
// packages@vassilijnadarajah.de

//! The Python module `bos_geo`. Compiled only with the `python` feature.
//!
//! The doc comments on the module function and on the two wrappers are the Python docstrings.
//! `bos_geo.pyi` repeats them for editors and type checkers; a test in tests/python compares the
//! two, so change them together.

use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;

use crate::metrics::{self, EndCap, MetricError};

/// Buffer overlay statistics for 2D polylines.
///
/// Scores a predicted line against a ground truth line with the buffer method of Tveite and
/// Langaas (1999): a part of one line counts as matched if it lies within a tolerance, the
/// buffer, of the other line.
///
/// calc_completeness (recall): the share of the ground truth that was found.
/// calc_correctness (precision): the share of the prediction that is right.
///
/// A line is a sequence of (x, y) pairs or an (N, 2) NumPy array, in planar coordinates.
#[pymodule]
fn bos_geo(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(calc_completeness_wrapper, m)?)?;
    m.add_function(wrap_pyfunction!(calc_correctness_wrapper, m)?)?;
    Ok(())
}

/// Completeness (recall): the share of the ground truth that was found.
///
/// Length(ground truth within the buffer of the prediction) / Length(ground truth)
///
/// Parameters:
///     prediction: The line being judged: its points in order, as a sequence of (x, y) pairs
///         or an (N, 2) NumPy array.
///     ground_truth: The line taken as correct, in the same form.
///     buffer: The tolerance, as a distance in the unit of the coordinates. A point of the
///         ground truth is found if it is at most this far from the prediction.
///     end_cap: Shape of the buffer at the two ends of the prediction. "round" (the default)
///         is everything within the buffer distance. "flat" cuts the buffer off at the ends:
///         ground truth that continues past an end of the prediction is not found there.
///
/// Returns:
///     A value between 0 and 1.
///
/// Raises:
///     ValueError: If a line has fewer than two distinct points or a coordinate that is NaN
///         or infinite, if buffer is not a finite number greater than 0, or if end_cap is
///         neither "round" nor "flat".
///
/// Coordinates are planar: both lines must be in the same projected coordinate system.
///
/// Example:
///     >>> import bos_geo
///     >>> prediction = [(0, 0), (4, 0)]
///     >>> ground_truth = [(0, 0), (8, 0)]
///     >>> bos_geo.calc_completeness(prediction, ground_truth, 0.5)
///     0.5625
///     >>> bos_geo.calc_completeness(prediction, ground_truth, 0.5, end_cap="flat")
///     0.5
#[pyfunction]
#[pyo3(name = "calc_completeness")]
#[pyo3(signature = (prediction, ground_truth, buffer, *, end_cap = "round"))]
fn calc_completeness_wrapper(
    prediction: Vec<[f64; 2]>,
    ground_truth: Vec<[f64; 2]>,
    buffer: f64,
    end_cap: &str,
) -> PyResult<f64> {
    let end_cap = parse_end_cap(end_cap)?;
    Ok(metrics::calc_completeness_with_end_cap(
        &prediction,
        &ground_truth,
        buffer,
        end_cap,
    )?)
}

/// Correctness (precision): the share of the prediction that is right.
///
/// Length(prediction within the buffer of the ground truth) / Length(prediction)
///
/// Parameters:
///     prediction: The line being judged: its points in order, as a sequence of (x, y) pairs
///         or an (N, 2) NumPy array.
///     ground_truth: The line taken as correct, in the same form.
///     buffer: The tolerance, as a distance in the unit of the coordinates. A point of the
///         prediction is right if it is at most this far from the ground truth.
///     end_cap: Shape of the buffer at the two ends of the ground truth. "round" (the default)
///         is everything within the buffer distance. "flat" cuts the buffer off at the ends:
///         a prediction that continues past an end of the ground truth is not right there.
///
/// Returns:
///     A value between 0 and 1.
///
/// Raises:
///     ValueError: If a line has fewer than two distinct points or a coordinate that is NaN
///         or infinite, if buffer is not a finite number greater than 0, or if end_cap is
///         neither "round" nor "flat".
///
/// Coordinates are planar: both lines must be in the same projected coordinate system.
///
/// Example:
///     >>> import bos_geo
///     >>> prediction = [(0, 0), (8, 0)]
///     >>> ground_truth = [(0, 0), (4, 0)]
///     >>> bos_geo.calc_correctness(prediction, ground_truth, 0.5)
///     0.5625
///     >>> bos_geo.calc_correctness(prediction, ground_truth, 0.5, end_cap="flat")
///     0.5
#[pyfunction]
#[pyo3(name = "calc_correctness")]
#[pyo3(signature = (prediction, ground_truth, buffer, *, end_cap = "round"))]
fn calc_correctness_wrapper(
    prediction: Vec<[f64; 2]>,
    ground_truth: Vec<[f64; 2]>,
    buffer: f64,
    end_cap: &str,
) -> PyResult<f64> {
    let end_cap = parse_end_cap(end_cap)?;
    Ok(metrics::calc_correctness_with_end_cap(
        &prediction,
        &ground_truth,
        buffer,
        end_cap,
    )?)
}

fn parse_end_cap(end_cap: &str) -> PyResult<EndCap> {
    match end_cap {
        "round" => Ok(EndCap::Round),
        "flat" => Ok(EndCap::Flat),
        other => Err(PyValueError::new_err(format!(
            "end_cap must be \"round\" or \"flat\", got {other:?}"
        ))),
    }
}

/// Unusable input is a `ValueError` in Python. The message starts with the name of the argument.
impl From<MetricError> for PyErr {
    fn from(error: MetricError) -> Self {
        PyValueError::new_err(error.to_string())
    }
}
