// SPDX-License-Identifier: GPL-3.0-only
// (C) 2025 Vassilij Nadarajah
// packages@vassilijnadarajah.de

//! Checks that hold for any valid input, on seeded random lines: agreement with a brute-force
//! reference, and the properties that follow from the definition of the two metrics. Plus a few
//! cases with hand-computed values, placed at arbitrary positions and angles.

mod common;

use bos_geo::EndCap;
use common::{
    Frame, Line, Rng, assert_close, completeness, completeness_with, correctness, correctness_with,
    densified, random_pair, random_segment, reference_share, reversed, rotated, scaled,
    shifted_sideways, translated,
};

const END_CAPS: [EndCap; 2] = [EndCap::Round, EndCap::Flat];

/// Rounding moves the input by a few ulps under the transformations below, and the result with it.
const TOLERANCE: f64 = 1e-9;

fn describe(case: usize, prediction: &Line, ground_truth: &Line, buffer: f64) -> String {
    format!(
        "case {case}, buffer {buffer}\nprediction {prediction:?}\nground truth {ground_truth:?}"
    )
}

#[test]
fn agrees_with_the_sampling_reference() {
    const SAMPLES: usize = 4000;
    let mut rng = Rng::new(10);
    let mut largest_difference: f64 = 0.0;
    for case in 0..400 {
        let (prediction, ground_truth, buffer) = random_pair(&mut rng, case);
        let what = describe(case, &prediction, &ground_truth, buffer);
        for end_cap in END_CAPS {
            let fast = completeness_with(&prediction, &ground_truth, buffer, end_cap);
            let slow = reference_share(&ground_truth, &prediction, buffer, end_cap, SAMPLES);
            assert_close(
                fast,
                slow.share,
                slow.resolution,
                &format!("completeness, {end_cap:?}, {what}"),
            );
            largest_difference = largest_difference.max((fast - slow.share).abs());

            let fast = correctness_with(&prediction, &ground_truth, buffer, end_cap);
            let slow = reference_share(&prediction, &ground_truth, buffer, end_cap, SAMPLES);
            assert_close(
                fast,
                slow.share,
                slow.resolution,
                &format!("correctness, {end_cap:?}, {what}"),
            );
            largest_difference = largest_difference.max((fast - slow.share).abs());
        }
    }
    println!("largest difference to the reference: {largest_difference:e}");
}

#[test]
fn result_is_a_share() {
    let mut rng = Rng::new(11);
    let mut partial = 0;
    for case in 0..2000 {
        let (prediction, ground_truth, buffer) = random_pair(&mut rng, case);
        let what = describe(case, &prediction, &ground_truth, buffer);
        for end_cap in END_CAPS {
            for value in [
                completeness_with(&prediction, &ground_truth, buffer, end_cap),
                correctness_with(&prediction, &ground_truth, buffer, end_cap),
            ] {
                assert!(
                    (0.0..=1.0).contains(&value),
                    "{value} is not in [0, 1]: {what}"
                );
                if value > 0.01 && value < 0.99 {
                    partial += 1;
                }
            }
        }
    }
    // The generator is only useful if most of its pairs are partly matched.
    assert!(
        partial > 4000,
        "only {partial} of 8000 results are strictly between 0 and 1"
    );
}

#[test]
fn completeness_is_correctness_with_the_lines_swapped() {
    let mut rng = Rng::new(12);
    for case in 0..500 {
        let (prediction, ground_truth, buffer) = random_pair(&mut rng, case);
        for end_cap in END_CAPS {
            assert_eq!(
                completeness_with(&prediction, &ground_truth, buffer, end_cap),
                correctness_with(&ground_truth, &prediction, buffer, end_cap),
                "{}",
                describe(case, &prediction, &ground_truth, buffer)
            );
        }
    }
}

#[test]
fn a_wider_buffer_never_matches_less() {
    let mut rng = Rng::new(13);
    for case in 0..500 {
        let (prediction, ground_truth, buffer) = random_pair(&mut rng, case);
        let wider = buffer * rng.uniform(1.0, 3.0);
        let what = describe(case, &prediction, &ground_truth, buffer);
        for end_cap in END_CAPS {
            let narrow = completeness_with(&prediction, &ground_truth, buffer, end_cap);
            let wide = completeness_with(&prediction, &ground_truth, wider, end_cap);
            assert!(
                narrow <= wide + 1e-12,
                "completeness {narrow} > {wide} at {wider}: {what}"
            );

            let narrow = correctness_with(&prediction, &ground_truth, buffer, end_cap);
            let wide = correctness_with(&prediction, &ground_truth, wider, end_cap);
            assert!(
                narrow <= wide + 1e-12,
                "correctness {narrow} > {wide} at {wider}: {what}"
            );
        }
    }
}

/// Asserts that both metrics, with both end caps, give the same values for the two pairs.
#[track_caller]
fn assert_same_result(
    original: (&Line, &Line, f64),
    changed: (&Line, &Line, f64),
    tolerance: f64,
    what: &str,
) {
    for end_cap in END_CAPS {
        assert_close(
            completeness_with(changed.0, changed.1, changed.2, end_cap),
            completeness_with(original.0, original.1, original.2, end_cap),
            tolerance,
            &format!("completeness, {end_cap:?}, {what}"),
        );
        assert_close(
            correctness_with(changed.0, changed.1, changed.2, end_cap),
            correctness_with(original.0, original.1, original.2, end_cap),
            tolerance,
            &format!("correctness, {end_cap:?}, {what}"),
        );
    }
}

#[test]
fn unchanged_by_translation_rotation_and_scaling() {
    let mut rng = Rng::new(14);
    for case in 0..500 {
        let (prediction, ground_truth, buffer) = random_pair(&mut rng, case);
        let original = (&prediction, &ground_truth, buffer);
        let what = describe(case, &prediction, &ground_truth, buffer);

        let (dx, dy) = (rng.uniform(-1e3, 1e3), rng.uniform(-1e3, 1e3));
        let moved = (
            &translated(&prediction, dx, dy),
            &translated(&ground_truth, dx, dy),
            buffer,
        );
        assert_same_result(original, moved, TOLERANCE, &format!("translation, {what}"));

        let angle = rng.uniform(0.0, std::f64::consts::TAU);
        let turned = (
            &rotated(&prediction, angle),
            &rotated(&ground_truth, angle),
            buffer,
        );
        assert_same_result(
            original,
            turned,
            TOLERANCE,
            &format!("rotation by {angle}, {what}"),
        );

        // From millimetres to kilometres in one go: coordinates and buffer together.
        let factor = 10_f64.powf(rng.uniform(-3.0, 3.0));
        let resized = (
            &scaled(&prediction, factor),
            &scaled(&ground_truth, factor),
            buffer * factor,
        );
        assert_same_result(
            original,
            resized,
            TOLERANCE,
            &format!("scaling by {factor}, {what}"),
        );
    }
}

#[test]
fn unchanged_by_the_direction_of_either_line() {
    let mut rng = Rng::new(15);
    for case in 0..500 {
        let (prediction, ground_truth, buffer) = random_pair(&mut rng, case);
        let original = (&prediction, &ground_truth, buffer);
        let what = describe(case, &prediction, &ground_truth, buffer);

        let changed = (&reversed(&prediction), &ground_truth, buffer);
        assert_same_result(
            original,
            changed,
            TOLERANCE,
            &format!("prediction reversed, {what}"),
        );
        let changed = (&prediction, &reversed(&ground_truth), buffer);
        assert_same_result(
            original,
            changed,
            TOLERANCE,
            &format!("ground truth reversed, {what}"),
        );
    }
}

#[test]
fn unchanged_by_extra_vertices_on_a_segment() {
    let mut rng = Rng::new(16);
    for case in 0..500 {
        let (prediction, ground_truth, buffer) = random_pair(&mut rng, case);
        let original = (&prediction, &ground_truth, buffer);
        let what = describe(case, &prediction, &ground_truth, buffer);

        let changed = (&densified(&mut rng, &prediction), &ground_truth, buffer);
        assert_same_result(
            original,
            changed,
            TOLERANCE,
            &format!("prediction densified, {what}"),
        );
        let changed = (&prediction, &densified(&mut rng, &ground_truth), buffer);
        assert_same_result(
            original,
            changed,
            TOLERANCE,
            &format!("ground truth densified, {what}"),
        );
    }
}

/// Two parallel segments of the same extent, a distance `d` apart, match fully if `d` is below the
/// buffer and not at all if it is above. The shape of the buffer ends plays no part here.
#[test]
fn parallel_lines_match_fully_or_not_at_all() {
    let mut rng = Rng::new(17);
    for case in 0..500 {
        let prediction = random_segment(&mut rng);
        let buffer = rng.uniform(0.05, 0.6);
        let what = format!("case {case}, buffer {buffer}, prediction {prediction:?}");

        let within = shifted_sideways(&prediction, buffer * rng.uniform(-0.99, 0.99));
        let beyond = shifted_sideways(&prediction, buffer * rng.uniform(1.01, 5.0));
        for end_cap in END_CAPS {
            assert_close(
                completeness_with(&prediction, &within, buffer, end_cap),
                1.0,
                TOLERANCE,
                &what,
            );
            assert_close(
                correctness_with(&prediction, &within, buffer, end_cap),
                1.0,
                TOLERANCE,
                &what,
            );
            assert_eq!(
                completeness_with(&prediction, &beyond, buffer, end_cap),
                0.0,
                "{what}"
            );
            assert_eq!(
                correctness_with(&prediction, &beyond, buffer, end_cap),
                0.0,
                "{what}"
            );
        }
    }
}

// ---------------------------------------------------------------------------------------------
// Hand-computed values
// ---------------------------------------------------------------------------------------------

/// One configuration in local coordinates, with the values worked out by hand.
struct Analytic {
    name: &'static str,
    prediction: Vec<(f64, f64)>,
    ground_truth: Vec<(f64, f64)>,
    buffer: f64,
    /// Expected (completeness, correctness) with round ends.
    round: (f64, f64),
    /// Expected (completeness, correctness) with flat ends.
    flat: (f64, f64),
}

fn analytic_cases() -> Vec<Analytic> {
    let sqrt2 = 2_f64.sqrt();
    vec![
        // Each line is within the buffer of the other for 2 · 0.25 of its length.
        Analytic {
            name: "crossing at a right angle",
            prediction: vec![(0.0, 0.0), (10.0, 0.0)],
            ground_truth: vec![(4.0, -3.0), (4.0, 5.0)],
            buffer: 0.25,
            round: (0.5 / 8.0, 0.5 / 10.0),
            flat: (0.5 / 8.0, 0.5 / 10.0),
        },
        // A crossing at 30° is inside the other buffer for 2 · 0.25 / sin(30°) = 1 of its length.
        Analytic {
            name: "crossing at 30 degrees",
            prediction: vec![(0.0, 0.0), (10.0, 0.0)],
            ground_truth: vec![
                (5.0 - 2.0 * 3_f64.sqrt(), -2.0),
                (5.0 + 3.0 * 3_f64.sqrt(), 3.0),
            ],
            buffer: 0.25,
            round: (1.0 / 10.0, 1.0 / 10.0),
            flat: (1.0 / 10.0, 1.0 / 10.0),
        },
        // The lines share the stretch from 2 to 6. A round end adds the buffer distance to it.
        Analytic {
            name: "collinear, overlapping by 4",
            prediction: vec![(0.0, 0.0), (6.0, 0.0)],
            ground_truth: vec![(2.0, 0.0), (9.0, 0.0)],
            buffer: 0.5,
            round: (4.5 / 7.0, 4.5 / 6.0),
            flat: (4.0 / 7.0, 4.0 / 6.0),
        },
        // Ground truth 0.15 beside the prediction and longer at both ends. A round end reaches
        // sqrt(0.25² - 0.15²) = 0.2 further along it.
        Analytic {
            name: "parallel at 0.15, ground truth longer",
            prediction: vec![(0.0, 0.0), (6.0, 0.0)],
            ground_truth: vec![(-2.0, 0.15), (8.0, 0.15)],
            buffer: 0.25,
            round: (6.4 / 10.0, 1.0),
            flat: (6.0 / 10.0, 1.0),
        },
        // The ground truth passes the outside of a right-angled corner at distance 1.2 / sqrt(2)
        // and crosses only the quarter disc there, in a chord of 2 · sqrt(1 - 0.72). Of the
        // prediction, the last sqrt(2) - 1.2 of each leg before the corner is within 1 of it.
        Analytic {
            name: "past the outside of a corner",
            prediction: vec![(-5.0, 0.0), (0.0, 0.0), (0.0, -5.0)],
            ground_truth: vec![(-0.8, 2.0), (2.0, -0.8)],
            buffer: 1.0,
            round: (
                2.0 * 0.28_f64.sqrt() / (2.8 * sqrt2),
                2.0 * (sqrt2 - 1.2) / 10.0,
            ),
            flat: (
                2.0 * 0.28_f64.sqrt() / (2.8 * sqrt2),
                2.0 * (sqrt2 - 1.2) / 10.0,
            ),
        },
        // A prediction doubling back 0.3 above itself. The ground truth runs between the two
        // legs and is longer by 1 at both ends. At the turn the buffer reaches 0.2 further; at
        // the open end only round ends add sqrt(0.2² - 0.15²).
        Analytic {
            name: "between the legs of a U-turn",
            prediction: vec![(0.0, 0.0), (10.0, 0.0), (10.0, 0.3), (0.0, 0.3)],
            ground_truth: vec![(-1.0, 0.15), (11.0, 0.15)],
            buffer: 0.2,
            round: ((10.2 + 0.0175_f64.sqrt()) / 12.0, 1.0),
            flat: (10.2 / 12.0, 1.0),
        },
    ]
}

#[test]
fn hand_computed_cases_at_arbitrary_angles() {
    let mut rng = Rng::new(18);
    for case in analytic_cases() {
        for _ in 0..50 {
            let frame = Frame::random(&mut rng);
            let prediction = frame.line(&case.prediction);
            let ground_truth = frame.line(&case.ground_truth);
            for (end_cap, expected) in [(EndCap::Round, case.round), (EndCap::Flat, case.flat)] {
                let what = format!("{}, {end_cap:?}", case.name);
                assert_close(
                    completeness_with(&prediction, &ground_truth, case.buffer, end_cap),
                    expected.0,
                    TOLERANCE,
                    &format!("completeness, {what}"),
                );
                assert_close(
                    correctness_with(&prediction, &ground_truth, case.buffer, end_cap),
                    expected.1,
                    TOLERANCE,
                    &format!("correctness, {what}"),
                );
            }
        }
    }
}

// ---------------------------------------------------------------------------------------------
// Large coordinates
// ---------------------------------------------------------------------------------------------

/// Lines in projected coordinates sit millions of units from the origin (a UTM northing in
/// central Europe is about 5.7 million). A coordinate of that size is stored to about 1e-9, so
/// the shifted input differs from the original by that much and no more can be asked of the
/// result than a tolerance to match. Without care, products of such coordinates lose far more.
#[test]
fn unchanged_at_utm_sized_coordinates() {
    let mut rng = Rng::new(19);
    for case in 0..500 {
        let (prediction, ground_truth, buffer) = random_pair(&mut rng, case);
        let original = (&prediction, &ground_truth, buffer);
        let (east, north) = (4.5e5, 5.7e6);
        let moved = (
            &translated(&prediction, east, north),
            &translated(&ground_truth, east, north),
            buffer,
        );
        let what = describe(case, &prediction, &ground_truth, buffer);
        assert_same_result(original, moved, 1e-6, &what);
    }
}

#[test]
fn hand_computed_cases_at_utm_sized_coordinates() {
    let frame = Frame::new([4.5e5, 5.7e6], 0.7);
    for case in analytic_cases() {
        let prediction = frame.line(&case.prediction);
        let ground_truth = frame.line(&case.ground_truth);
        assert_close(
            completeness(&prediction, &ground_truth, case.buffer),
            case.round.0,
            1e-6,
            case.name,
        );
        assert_close(
            correctness(&prediction, &ground_truth, case.buffer),
            case.round.1,
            1e-6,
            case.name,
        );
    }
}
