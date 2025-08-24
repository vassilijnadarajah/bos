// SPDX-License-Identifier: GPL-3.0-only
// (C) 2025 Vassilij Nadarajah
// packages@vassilijnadarajah.de

pub mod point2d;
pub mod geometry;

use point2d::AsPoint2D;
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
    let xb = match create_line_buffer(prediction, buffer) {
        Some(buffer) => buffer,
        None => return 0.0, // no buffer => no completeness
    };

    // Step 2: overlay ground truth (Q) with buffered prediction (XB) and get length of QXB
    let qxb_len = calc_line_polygon_overlay(ground_truth, &xb);

    // Step 3: compute completeness
    qxb_len / calc_line_length(ground_truth)
}

#[cfg(test)]
mod tests {
    use crate::point2d::Point2D;
    use super::*;

    #[test]
    fn test_completeness() {
        let prediction = vec![
            Point2D::new(-1, -2),
            Point2D::new(-1, 0),
            Point2D::new(0, 1),
            Point2D::new(2, 1)
        ];
        let ground_truth = vec![
            Point2D::new(-1, -2),
            Point2D::new(-1, 1),
            Point2D::new(1, 1),
            Point2D::new(2, 1),
        ];
        let buffer = 0.2;

        let result = calc_completeness(&prediction, &ground_truth, buffer);
        let completeness = 2.0 * 2.0_f64.sqrt() * buffer + 2.0;

        assert!(
            (result - completeness).abs() < 1e-10,
            "Completeness is {} but should be {}",
            result, completeness
        );
    }
}