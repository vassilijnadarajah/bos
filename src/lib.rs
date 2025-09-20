// SPDX-License-Identifier: GPL-3.0-only
// (C) 2025 Vassilij Nadarajah
// packages@vassilijnadarajah.de

pub mod point2d;
pub mod geometry;
pub mod metrics;

use pyo3::prelude::*;

#[pymodule]
fn bos(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(calc_correctness_wrapper, m)?)?;
    m.add_function(wrap_pyfunction!(calc_completeness_wrapper, m)?)?;
    Ok(())
}

#[pyfunction]
#[pyo3(name = "calc_completeness")]
#[pyo3(signature = (prediction, ground_truth, buffer))]
fn calc_completeness_wrapper(prediction: Vec<[f64; 2]>, ground_truth: Vec<[f64; 2]>, buffer: f64) -> f64 {
    metrics::calc_completeness(&prediction, &ground_truth, buffer)
}

#[pyfunction]
#[pyo3(name = "calc_correctness")]
#[pyo3(signature = (prediction, ground_truth, buffer))]
fn calc_correctness_wrapper(prediction: Vec<[f64; 2]>, ground_truth: Vec<[f64; 2]>, buffer: f64) -> f64 {
    metrics::calc_correctness(&prediction, &ground_truth, buffer)
}