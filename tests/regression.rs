// SPDX-License-Identifier: GPL-3.0-only
// (C) 2025 Vassilij Nadarajah
// packages@vassilijnadarajah.de

//! Inputs on which the metrics have returned wrong values, each with the value that the method
//! defines: the share of one line that lies within the buffer distance of the other.

mod common;

use common::{
    Line, Rng, assert_close, completeness, correctness, random_polyline, random_segment,
    shifted_sideways,
};

const TOLERANCE: f64 = 1e-9;

/// A line lies within any buffer around itself. Its end points are exactly on the ends of that
/// buffer, so this used to depend on rounding and came out as 0 for half of all segments.
#[test]
#[ignore = "known defect of the polygon overlay; fixed by replacing the overlay core"]
fn segment_scored_against_itself_is_fully_matched() {
    let mut rng = Rng::new(1);
    for case in 0..1000 {
        let line = random_segment(&mut rng);
        for buffer in [0.05, 0.5] {
            let what = format!("case {case}, buffer {buffer}, line {line:?}");
            assert_close(completeness(&line, &line, buffer), 1.0, TOLERANCE, &what);
            assert_close(correctness(&line, &line, buffer), 1.0, TOLERANCE, &what);
        }
    }
}

/// The same for polylines, half of them gentle curves and half with sharp turns and loops.
#[test]
#[ignore = "known defect of the polygon overlay; fixed by replacing the overlay core"]
fn polyline_scored_against_itself_is_fully_matched() {
    let mut rng = Rng::new(2);
    for case in 0..1000 {
        let vertices = 3 + rng.index(20);
        let max_turn = if case % 2 == 0 {
            0.3
        } else {
            std::f64::consts::PI
        };
        let line = random_polyline(&mut rng, vertices, max_turn);
        for buffer in [0.05, 0.5] {
            let what = format!("case {case}, buffer {buffer}, line {line:?}");
            assert_close(completeness(&line, &line, buffer), 1.0, TOLERANCE, &what);
            assert_close(correctness(&line, &line, buffer), 1.0, TOLERANCE, &what);
        }
    }
}

/// A ground truth that is the prediction moved sideways by less than the buffer is fully matched.
/// The shift puts the end points of each line exactly on the end of the other line's buffer.
#[test]
#[ignore = "known defect of the polygon overlay; fixed by replacing the overlay core"]
fn sideways_shift_within_the_buffer_is_fully_matched() {
    let mut rng = Rng::new(3);
    for case in 0..1000 {
        let prediction = random_segment(&mut rng);
        for buffer in [0.05, 0.5] {
            let ground_truth = shifted_sideways(&prediction, 0.3 * buffer);
            let what = format!("case {case}, buffer {buffer}, prediction {prediction:?}");
            assert_close(
                completeness(&prediction, &ground_truth, buffer),
                1.0,
                TOLERANCE,
                &what,
            );
            assert_close(
                correctness(&prediction, &ground_truth, buffer),
                1.0,
                TOLERANCE,
                &what,
            );
        }
    }
}

/// A vertex listed twice carries no information. It used to pinch the buffer to zero width there.
#[test]
#[ignore = "known defect of the polygon overlay; fixed by replacing the overlay core"]
fn duplicate_vertex_does_not_pinch_the_buffer() {
    let with_duplicate: Line = vec![[0.0, 0.0], [1.0, 0.0], [1.0, 0.0], [2.0, 0.0]];
    let other: Line = vec![[0.0, 0.1], [2.0, 0.1]];
    assert_close(
        completeness(&with_duplicate, &other, 0.2),
        1.0,
        TOLERANCE,
        "completeness",
    );
    assert_close(
        correctness(&other, &with_duplicate, 0.2),
        1.0,
        TOLERANCE,
        "correctness",
    );
}

/// The buffer ends at the buffer distance from a sharp turn. A mitre join used to carry it 20
/// units past this hairpin, far enough to cover a ground truth that is 5 units away from it.
#[test]
#[ignore = "known defect of the polygon overlay; fixed by replacing the overlay core"]
fn sharp_turn_does_not_extend_the_buffer() {
    let prediction: Line = vec![[0.0, 0.0], [10.0, 0.0], [0.0, 0.5]];
    let ground_truth: Line = vec![[15.0, -0.3], [25.0, -0.3]];
    assert_close(
        completeness(&prediction, &ground_truth, 0.5),
        0.0,
        TOLERANCE,
        "completeness",
    );
}

/// Where the buffers of two parts of a line overlap, the overlap still belongs to the buffer. The
/// even-odd rule used to count the doubly covered strip between the two legs as outside.
#[test]
#[ignore = "known defect of the polygon overlay; fixed by replacing the overlay core"]
fn line_doubling_back_keeps_the_strip_between_its_legs() {
    let prediction: Line = vec![[0.0, 0.0], [10.0, 0.0], [10.0, 0.3], [0.0, 0.3]];
    let ground_truth: Line = vec![[1.0, 0.15], [9.0, 0.15]];
    assert_close(
        completeness(&prediction, &ground_truth, 0.2),
        1.0,
        TOLERANCE,
        "completeness",
    );
}

/// The buffer is everything within the buffer distance, so it reaches past the ends of the line.
#[test]
#[ignore = "known defect of the polygon overlay; fixed by replacing the overlay core"]
fn buffer_reaches_past_the_line_ends() {
    let prediction: Line = vec![[0.0, 0.0], [2.0, 0.0]];
    let ground_truth: Line = vec![[-0.05, 0.0], [2.05, 0.0]];
    assert_close(
        completeness(&prediction, &ground_truth, 0.1),
        1.0,
        TOLERANCE,
        "completeness",
    );
}

/// A closed ring against the same ring drawn 0.05 further in.
#[test]
#[ignore = "known defect of the polygon overlay; fixed by replacing the overlay core"]
fn closed_ring_is_fully_matched() {
    let outer: Line = vec![
        [0.0, 0.0],
        [10.0, 0.0],
        [10.0, 10.0],
        [0.0, 10.0],
        [0.0, 0.0],
    ];
    let inner: Line = vec![
        [0.05, 0.05],
        [9.95, 0.05],
        [9.95, 9.95],
        [0.05, 9.95],
        [0.05, 0.05],
    ];
    assert_close(
        completeness(&outer, &inner, 0.2),
        1.0,
        TOLERANCE,
        "completeness",
    );
    assert_close(
        correctness(&outer, &inner, 0.2),
        1.0,
        TOLERANCE,
        "correctness",
    );
}

/// A straight line with many vertices is the same line as its two end points. The mitre join of
/// two almost parallel offset lines is ill-conditioned and has misplaced buffer vertices by
/// several units on input like this.
#[test]
#[ignore = "known defect of the polygon overlay; fixed by replacing the overlay core"]
fn dense_collinear_line_scores_like_its_two_end_points() {
    let mut rng = Rng::new(4);
    for case in 0..200 {
        let heading = rng.uniform(0.0, std::f64::consts::TAU);
        let (dx, dy) = (heading.cos(), heading.sin());
        let origin = [rng.uniform(-50.0, 50.0), rng.uniform(-50.0, 50.0)];
        // A point `along` the line and `across` to its left.
        let at = |along: f64, across: f64| {
            [
                origin[0] + along * dx - across * dy,
                origin[1] + along * dy + across * dx,
            ]
        };

        let dense: Line = (0..40)
            .map(|i| {
                let [x, y] = at(0.25 * i as f64, 0.0);
                [x + rng.uniform(0.0, 1e-13), y + rng.uniform(0.0, 1e-13)]
            })
            .collect();
        let end_points: Line = vec![dense[0], dense[39]];
        // Runs beside the line, then leaves it.
        let other: Line = vec![at(2.0, 0.05), at(6.0, 0.05), at(9.0, 1.0)];

        let buffer = 0.1;
        let what = format!("case {case}, heading {heading}");
        assert_close(
            completeness(&dense, &other, buffer),
            completeness(&end_points, &other, buffer),
            TOLERANCE,
            &what,
        );
        assert_close(
            correctness(&dense, &other, buffer),
            correctness(&end_points, &other, buffer),
            TOLERANCE,
            &what,
        );
        assert_close(
            completeness(&other, &dense, buffer),
            completeness(&other, &end_points, buffer),
            TOLERANCE,
            &what,
        );
        assert_close(
            correctness(&other, &dense, buffer),
            correctness(&other, &end_points, buffer),
            TOLERANCE,
            &what,
        );
    }
}
