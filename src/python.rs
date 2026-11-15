// SPDX-License-Identifier: GPL-3.0-only
// (C) 2025 Vassilij Nadarajah
// packages@vassilijnadarajah.de

use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;

use crate::geometry::EndCap;
use crate::metrics::{self, MetricError};

#[pymodule]
fn bos_geo(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(calc_correctness_wrapper, m)?)?;
    m.add_function(wrap_pyfunction!(calc_completeness_wrapper, m)?)?;
    Ok(())
}

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
