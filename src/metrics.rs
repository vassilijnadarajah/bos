// SPDX-License-Identifier: GPL-3.0-only
// (C) 2025 Vassilij Nadarajah
// packages@vassilijnadarajah.de

use crate::geometry::{EndCap, covered_fraction};
use crate::point2d::{AsPoint2D, Point2D};

/// Share of the ground truth that lies within `buffer` of the prediction.
pub fn calc_completeness<T: AsPoint2D + Copy>(
    prediction: &Vec<T>,
    ground_truth: &Vec<T>,
    buffer: f64,
) -> f64 {
    calc_completeness_with_end_cap(prediction, ground_truth, buffer, EndCap::default())
}

/// Like [`calc_completeness`], with a choice of the buffer's shape at the ends of the prediction.
pub fn calc_completeness_with_end_cap<T: AsPoint2D + Copy>(
    prediction: &Vec<T>,
    ground_truth: &Vec<T>,
    buffer: f64,
    end_cap: EndCap,
) -> f64 {
    // Length(ground truth inside buffer(prediction)) / Length(ground truth)
    share_within_buffer(ground_truth, prediction, buffer, end_cap)
}

/// Share of the prediction that lies within `buffer` of the ground truth.
pub fn calc_correctness<T: AsPoint2D + Copy>(
    prediction: &Vec<T>,
    ground_truth: &Vec<T>,
    buffer: f64,
) -> f64 {
    calc_correctness_with_end_cap(prediction, ground_truth, buffer, EndCap::default())
}

/// Like [`calc_correctness`], with a choice of the buffer's shape at the ends of the ground truth.
pub fn calc_correctness_with_end_cap<T: AsPoint2D + Copy>(
    prediction: &Vec<T>,
    ground_truth: &Vec<T>,
    buffer: f64,
    end_cap: EndCap,
) -> f64 {
    // Length(prediction inside buffer(ground truth)) / Length(prediction)
    share_within_buffer(prediction, ground_truth, buffer, end_cap)
}

/// Length of `line` inside the buffer of `buffered`, divided by the length of `line`.
fn share_within_buffer<T: AsPoint2D + Copy>(
    line: &Vec<T>,
    buffered: &Vec<T>,
    buffer: f64,
    end_cap: EndCap,
) -> f64 {
    if buffered.len() < 2 {
        return 0.0; // no buffer => nothing inside it
    }
    let line: Vec<Point2D> = line.iter().map(AsPoint2D::to_point2d).collect();
    let buffered: Vec<Point2D> = buffered.iter().map(AsPoint2D::to_point2d).collect();
    covered_fraction(&line, &buffered, buffer.abs(), end_cap).unwrap_or(f64::NAN)
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
        let result = calc_completeness(&prediction, &ground_truth, buffer);
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
            calc_completeness_with_end_cap(&prediction, &ground_truth, buffer, EndCap::Flat);
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
                calc_completeness_with_end_cap(&prediction, &ground_truth, buffer, end_cap);
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

        let result = calc_correctness(&prediction, &ground_truth, buffer);
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
            calc_correctness_with_end_cap(&prediction, &ground_truth, buffer, EndCap::Flat);
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
            let result = calc_correctness_with_end_cap(&prediction, &ground_truth, buffer, end_cap);
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
