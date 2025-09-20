// SPDX-License-Identifier: GPL-3.0-only
// (C) 2025 Vassilij Nadarajah
// packages@vassilijnadarajah.de


use crate::point2d::AsPoint2D;
use crate::geometry::{create_line_buffer, calc_line_polygon_overlay, calc_line_length};

pub fn calc_completeness<T: AsPoint2D + Copy>(prediction: &Vec<T>, ground_truth: &Vec<T>, buffer: f64) -> f64 {
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

pub fn calc_correctness<T: AsPoint2D + Copy>(prediction: &Vec<T>, ground_truth: &Vec<T>, buffer: f64) -> f64 {
    // Step 1: line buffering of ground truth (QB)
    let qb = match create_line_buffer(ground_truth, buffer) {
        Some(buffer) => buffer,
        None => return 0.0, // no buffer => no correctness
    };

    // Step 2: overlay prediction (X) with buffered ground truth (QB) and get length of XQB
    let xqb_len = calc_line_polygon_overlay(prediction, &qb);

    // Step 3: compute correctness
    xqb_len / calc_line_length(prediction)
}

#[cfg(test)]
mod tests {
    use crate::point2d::Point2D;
    use super::*;

    #[test]
    fn test_completeness_simple() {
        let prediction = vec![
            Point2D::new(0, 0),
            Point2D::new(2, 2),
        ];
        let ground_truth = vec![
            Point2D::new(1, 1),
            Point2D::new(3, 3),
        ];
        let buffer = 0.1;

        let result = calc_completeness(&prediction, &ground_truth, buffer);
        let completeness = 0.5;
        assert!(
            (result - completeness).abs() < 1e-10,
            "Completeness is {} but should be {}",
            result, completeness
        );
    }

    #[test]
    fn test_completeness_complex() {
        let prediction = vec![
            Point2D::new(-1, -2),
            Point2D::new(-1, 0),
            Point2D::new(0, 1),
            Point2D::new(2, 1)
        ];
        let ground_truth = vec![
            Point2D::new(-1, -2),
            Point2D::new(-1, -1),
            Point2D::new(1, 1),
            Point2D::new(2, 1),
        ];
        let buffer = 0.2;

        let result = calc_completeness(&prediction, &ground_truth, buffer);
        let completeness = (2.0 * 2_f64.sqrt() * buffer + 2.0) / (2.0 * 2_f64.sqrt() + 2.0);

        assert!(
            (result - completeness).abs() < 1e-10,
            "Completeness is {} but should be {}",
            result, completeness
        );
    }

    #[test]
    fn test_correctness_simple() {
        let prediction = vec![
            Point2D::new(0, 0),
            Point2D::new(2, 2),
        ];
        let ground_truth = vec![
            Point2D::new(1, 1),
            Point2D::new(3, 3),
        ];
        let buffer = 0.1;

        let result = calc_correctness(&prediction, &ground_truth, buffer);
        let correctness = 0.5;
        assert!(
            (result - correctness).abs() < 1e-10,
            "Correctness is {} but should be {}",
            result, correctness
        );
    }

        #[test]
    fn test_correctness_complex() {
        let prediction = vec![
            Point2D::new(-1, -2),
            Point2D::new(-1, 0),
            Point2D::new(0, 1),
            Point2D::new(2, 1)
        ];
        let ground_truth = vec![
            Point2D::new(-1, -2),
            Point2D::new(-1, -1),
            Point2D::new(1, 1),
            Point2D::new(2, 1),
        ];
        let buffer = 0.2;

        let result = calc_correctness(&prediction, &ground_truth, buffer);
        let correctness = (2.0 * 2_f64.sqrt() * buffer + 2.0) / (2_f64.sqrt() + 4.0);

        assert!(
            (result - correctness).abs() < 1e-10,
            "Correctness is {} but should be {}",
            result, correctness
        );
    }
}