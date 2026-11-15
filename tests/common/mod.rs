// SPDX-License-Identifier: GPL-3.0-only
// (C) 2025 Vassilij Nadarajah
// packages@vassilijnadarajah.de

//! Helpers shared by the integration tests: a seeded random number generator, line generators and
//! thin wrappers around the two metrics.

// Each test binary uses its own subset of these helpers.
#![allow(dead_code)]

/// A polyline as the tests pass it to the library.
pub type Line = Vec<[f64; 2]>;

/// SplitMix64. Written out here so the tests need no dependency and see the same input on every
/// run and every platform.
pub struct Rng(u64);

impl Rng {
    pub fn new(seed: u64) -> Self {
        Rng(seed)
    }

    pub fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// Uniform in `[0, 1)`.
    pub fn unit(&mut self) -> f64 {
        (self.next_u64() >> 11) as f64 / (1u64 << 53) as f64
    }

    /// Uniform in `[lo, hi)`.
    pub fn uniform(&mut self, lo: f64, hi: f64) -> f64 {
        lo + (hi - lo) * self.unit()
    }

    /// Uniform in `0..n`.
    pub fn index(&mut self, n: usize) -> usize {
        (self.next_u64() % n as u64) as usize
    }
}

/// A segment in general position: both end points uniform in a 100 × 100 square, at least 1 apart.
pub fn random_segment(rng: &mut Rng) -> Line {
    loop {
        let a = [rng.uniform(-50.0, 50.0), rng.uniform(-50.0, 50.0)];
        let b = [rng.uniform(-50.0, 50.0), rng.uniform(-50.0, 50.0)];
        if (b[0] - a[0]).hypot(b[1] - a[1]) >= 1.0 {
            return vec![a, b];
        }
    }
}

/// A random walk with `vertices` vertices, steps of 0.5 to 3 and a heading change of at most
/// `max_turn` radians per vertex. A small `max_turn` gives gentle curves; `PI` gives sharp turns,
/// loops and self-crossings.
pub fn random_polyline(rng: &mut Rng, vertices: usize, max_turn: f64) -> Line {
    let mut point = [rng.uniform(-50.0, 50.0), rng.uniform(-50.0, 50.0)];
    let mut heading = rng.uniform(0.0, std::f64::consts::TAU);
    let mut line = vec![point];
    for _ in 1..vertices {
        heading += rng.uniform(-max_turn, max_turn);
        let step = rng.uniform(0.5, 3.0);
        point = [
            point[0] + step * heading.cos(),
            point[1] + step * heading.sin(),
        ];
        line.push(point);
    }
    line
}

/// `line` moved by `distance` to the left of the direction from its first to its last vertex.
pub fn shifted_sideways(line: &Line, distance: f64) -> Line {
    let (first, last) = (line[0], line[line.len() - 1]);
    let (dx, dy) = (last[0] - first[0], last[1] - first[1]);
    let length = dx.hypot(dy);
    let (nx, ny) = (-dy / length * distance, dx / length * distance);
    line.iter().map(|p| [p[0] + nx, p[1] + ny]).collect()
}

pub fn completeness(prediction: &Line, ground_truth: &Line, buffer: f64) -> f64 {
    bos_geo::metrics::calc_completeness(prediction, ground_truth, buffer)
}

pub fn correctness(prediction: &Line, ground_truth: &Line, buffer: f64) -> f64 {
    bos_geo::metrics::calc_correctness(prediction, ground_truth, buffer)
}

#[track_caller]
pub fn assert_close(actual: f64, expected: f64, tolerance: f64, what: &str) {
    assert!(
        (actual - expected).abs() <= tolerance,
        "{what}: got {actual}, expected {expected} (tolerance {tolerance})"
    );
}
