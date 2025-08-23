// SPDX-License-Identifier: GPL-3.0-only
// (C) 2025 Vassilij Nadarajah
// packages@vassilijnadarajah.de

pub mod point2d;
pub mod geometry;

use point2d::{Point2D, AsPoint2D};
use geometry::{create_line_buffer, calc_line_polygon_overlay, calc_line_length};
use pyo3::prelude::*;

#[pymodule]
fn bos(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(calc_completeness_wrapper, m)?)?;
    Ok(())
}

#[pyfunction]
#[pyo3(name = "calc_completeness")]
#[pyo3(signature = (prediction, ground_truth, buffer))]
fn calc_completeness_wrapper(prediction: Vec<[f64; 2]>, ground_truth: Vec<[f64; 2]>, buffer: f64) -> f64 {
    calc_completeness(&prediction, &ground_truth, buffer)
}

fn calc_completeness<T: AsPoint2D + Copy>(prediction: &Vec<T>, ground_truth: &Vec<T>, buffer: f64) -> f64 {
    // Step 1: line buffering of prediction (XB)
    let xb = create_line_buffer(prediction, buffer).unwrap();

    // Step 2: overlay ground truth (Q) with buffered prediction (XB) and get length of QXB
    let qxb_len = calc_line_polygon_overlay(ground_truth, &xb);

    // Step 3: compute completeness
    qxb_len / calc_line_length(ground_truth)
}

pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}