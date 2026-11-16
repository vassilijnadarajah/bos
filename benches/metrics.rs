// SPDX-License-Identifier: GPL-3.0-only
// (C) 2025 Vassilij Nadarajah
// packages@vassilijnadarajah.de

//! Run time of one metric call on lines of 1 000 to 20 000 vertices. Run it with `cargo bench`.
//!
//! The input resembles a detected curb line and its reference: a gently curving ground truth with
//! a vertex every 10 cm, a prediction that is the ground truth with up to 12 cm of noise in each
//! coordinate, and a buffer of 10 cm, so that most but not all of each line is matched.
//!
//! There is no benchmark framework behind this, to keep the crate free of dependencies. Each
//! call is repeated for about a second and the fastest and the median run are printed.

use std::hint::black_box;
use std::time::{Duration, Instant};

use bos_geo::{calc_completeness, calc_correctness};

type Line = Vec<[f64; 2]>;

const SIZES: [usize; 3] = [1_000, 5_000, 20_000];
const BUFFER: f64 = 0.1;

fn main() {
    // `cargo bench` passes `--bench`. Without it, for example under `cargo test --benches`, run
    // each call once on a small input to show that the benchmark still works.
    let measure = std::env::args().any(|argument| argument == "--bench");
    let sizes: &[usize] = if measure { &SIZES } else { &[200] };

    println!(
        "{:>9}  {:<12}  {:>10}  {:>10}  {:>6}",
        "vertices", "metric", "fastest", "median", "value"
    );
    for &vertices in sizes {
        let (prediction, ground_truth) = lines(vertices);
        let completeness = || calc_completeness(&prediction, &ground_truth, BUFFER).unwrap();
        let correctness = || calc_correctness(&prediction, &ground_truth, BUFFER).unwrap();
        report(vertices, "completeness", measure, completeness);
        report(vertices, "correctness", measure, correctness);
    }
}

fn report(vertices: usize, metric: &str, measure: bool, mut call: impl FnMut() -> f64) {
    let budget = if measure {
        Duration::from_secs(1)
    } else {
        Duration::ZERO
    };
    let mut value = 0.0;
    let mut runs = Vec::new();
    let started = Instant::now();
    while runs.is_empty() || (runs.len() < 200 && started.elapsed() < budget) {
        let run = Instant::now();
        value = black_box(call());
        runs.push(run.elapsed());
    }
    runs.sort();
    println!(
        "{vertices:>9}  {metric:<12}  {:>7.2} ms  {:>7.2} ms  {value:>6.3}",
        milliseconds(runs[0]),
        milliseconds(runs[runs.len() / 2]),
    );
}

fn milliseconds(duration: Duration) -> f64 {
    duration.as_secs_f64() * 1e3
}

/// A prediction and a ground truth with the given number of vertices each.
fn lines(vertices: usize) -> (Line, Line) {
    let mut rng = Rng(vertices as u64);
    let mut point = [0.0_f64, 0.0];
    let mut heading = 0.0_f64;
    let mut ground_truth = Vec::with_capacity(vertices);
    for _ in 0..vertices {
        ground_truth.push(point);
        // Up to 4.6° of heading change per metre.
        heading += rng.uniform(-0.008, 0.008);
        point = [
            point[0] + 0.1 * heading.cos(),
            point[1] + 0.1 * heading.sin(),
        ];
    }
    let prediction = ground_truth
        .iter()
        .map(|p| {
            [
                p[0] + rng.uniform(-0.12, 0.12),
                p[1] + rng.uniform(-0.12, 0.12),
            ]
        })
        .collect();
    (prediction, ground_truth)
}

/// SplitMix64, so that every run measures the same input.
struct Rng(u64);

impl Rng {
    fn uniform(&mut self, low: f64, high: f64) -> f64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^= z >> 31;
        low + (high - low) * ((z >> 11) as f64 / (1u64 << 53) as f64)
    }
}
