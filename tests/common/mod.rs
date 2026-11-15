// SPDX-License-Identifier: GPL-3.0-only
// (C) 2025 Vassilij Nadarajah
// packages@vassilijnadarajah.de

//! Helpers shared by the integration tests: a seeded random number generator, line generators and
//! thin wrappers around the two metrics.

// Each test binary uses its own subset of these helpers.
#![allow(dead_code)]

use bos_geo::geometry::EndCap;

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
    bos_geo::metrics::calc_completeness(prediction, ground_truth, buffer).expect("valid input")
}

pub fn correctness(prediction: &Line, ground_truth: &Line, buffer: f64) -> f64 {
    bos_geo::metrics::calc_correctness(prediction, ground_truth, buffer).expect("valid input")
}

pub fn completeness_with(
    prediction: &Line,
    ground_truth: &Line,
    buffer: f64,
    end_cap: EndCap,
) -> f64 {
    bos_geo::metrics::calc_completeness_with_end_cap(prediction, ground_truth, buffer, end_cap)
        .expect("valid input")
}

pub fn correctness_with(
    prediction: &Line,
    ground_truth: &Line,
    buffer: f64,
    end_cap: EndCap,
) -> f64 {
    bos_geo::metrics::calc_correctness_with_end_cap(prediction, ground_truth, buffer, end_cap)
        .expect("valid input")
}

#[track_caller]
pub fn assert_close(actual: f64, expected: f64, tolerance: f64, what: &str) {
    assert!(
        (actual - expected).abs() <= tolerance,
        "{what}: got {actual}, expected {expected} (tolerance {tolerance})"
    );
}

// ---------------------------------------------------------------------------------------------
// Transformations
// ---------------------------------------------------------------------------------------------

pub fn translated(line: &Line, dx: f64, dy: f64) -> Line {
    line.iter().map(|p| [p[0] + dx, p[1] + dy]).collect()
}

/// `line` rotated by `angle` radians about the origin.
pub fn rotated(line: &Line, angle: f64) -> Line {
    let (sin, cos) = angle.sin_cos();
    line.iter()
        .map(|p| [p[0] * cos - p[1] * sin, p[0] * sin + p[1] * cos])
        .collect()
}

pub fn scaled(line: &Line, factor: f64) -> Line {
    line.iter()
        .map(|p| [p[0] * factor, p[1] * factor])
        .collect()
}

pub fn reversed(line: &Line) -> Line {
    line.iter().rev().copied().collect()
}

/// `line` with up to three extra vertices on each of its segments. The shape is unchanged.
pub fn densified(rng: &mut Rng, line: &Line) -> Line {
    let mut dense = vec![line[0]];
    for pair in line.windows(2) {
        let (a, b) = (pair[0], pair[1]);
        let mut fractions: Vec<f64> = (0..rng.index(4)).map(|_| rng.unit()).collect();
        fractions.sort_by(f64::total_cmp);
        for t in fractions {
            dense.push([a[0] + t * (b[0] - a[0]), a[1] + t * (b[1] - a[1])]);
        }
        dense.push(b);
    }
    dense
}

/// A local coordinate system at an arbitrary position and angle. Test cases are written in local
/// coordinates, where the expected value is easy to work out by hand, and evaluated in world
/// coordinates, where nothing is axis-aligned.
pub struct Frame {
    origin: [f64; 2],
    sin: f64,
    cos: f64,
}

impl Frame {
    pub fn new(origin: [f64; 2], angle: f64) -> Self {
        let (sin, cos) = angle.sin_cos();
        Frame { origin, sin, cos }
    }

    pub fn random(rng: &mut Rng) -> Self {
        Frame::new(
            [rng.uniform(-500.0, 500.0), rng.uniform(-500.0, 500.0)],
            rng.uniform(0.0, std::f64::consts::TAU),
        )
    }

    /// The world coordinates of the local point `(x, y)`.
    pub fn point(&self, x: f64, y: f64) -> [f64; 2] {
        [
            self.origin[0] + x * self.cos - y * self.sin,
            self.origin[1] + x * self.sin + y * self.cos,
        ]
    }

    pub fn line(&self, local: &[(f64, f64)]) -> Line {
        local.iter().map(|&(x, y)| self.point(x, y)).collect()
    }
}

// ---------------------------------------------------------------------------------------------
// Reference implementation
// ---------------------------------------------------------------------------------------------

/// What the sampling reference found.
pub struct Reference {
    /// Share of the line's sample points that are in the buffer.
    pub share: f64,
    /// How far `share` can be from the exact value: half a sample spacing for every place where
    /// consecutive samples change between inside and outside, plus two spacings for anything
    /// narrower than the spacing.
    pub resolution: f64,
}

/// The share of `line` in the buffer of `buffered`, by brute force: sample `line` densely and test
/// every sample against the definition of the buffer. Slow, and exact only up to the sample
/// spacing, but too simple to be wrong in the ways the fast code could be.
pub fn reference_share(
    line: &Line,
    buffered: &Line,
    buffer: f64,
    end_cap: EndCap,
    samples: usize,
) -> Reference {
    let total: f64 = line.windows(2).map(|pair| distance(pair[0], pair[1])).sum();
    let spacing = total / samples as f64;

    let mut covered = 0.0;
    let mut changes = 0;
    let mut previous = None;
    for pair in line.windows(2) {
        let length = distance(pair[0], pair[1]);
        let steps = (length / spacing).ceil().max(1.0) as usize;
        for step in 0..steps {
            let t = (step as f64 + 0.5) / steps as f64;
            let sample = [
                pair[0][0] + t * (pair[1][0] - pair[0][0]),
                pair[0][1] + t * (pair[1][1] - pair[0][1]),
            ];
            let inside = in_buffer(sample, buffered, buffer, end_cap);
            if inside {
                covered += length / steps as f64;
            }
            if previous.is_some_and(|previous| previous != inside) {
                changes += 1;
            }
            previous = Some(inside);
        }
    }

    Reference {
        share: covered / total,
        resolution: (0.5 * changes as f64 + 2.0) / samples as f64,
    }
}

/// Whether `point` is in the buffer of `line`, straight from the definition.
fn in_buffer(point: [f64; 2], line: &Line, buffer: f64, end_cap: EndCap) -> bool {
    match end_cap {
        // Within the buffer distance of the line.
        EndCap::Round => line
            .windows(2)
            .any(|pair| distance_to_segment(point, pair[0], pair[1]) <= buffer),
        // The same, except that being close to one of the two ends of the line does not count.
        // What is left is a point beside a segment (its foot is on the segment), or around the
        // outside of a bend (past the end of one segment and before the start of the next).
        EndCap::Flat => {
            let beside_a_segment = line.windows(2).any(|pair| {
                let foot = foot_parameter(point, pair[0], pair[1]);
                (0.0..=1.0).contains(&foot)
                    && distance_to_segment(point, pair[0], pair[1]) <= buffer
            });
            let around_a_bend = line.windows(3).any(|triple| {
                distance(point, triple[1]) <= buffer
                    && foot_parameter(point, triple[0], triple[1]) >= 1.0
                    && foot_parameter(point, triple[1], triple[2]) <= 0.0
            });
            beside_a_segment || around_a_bend
        }
    }
}

fn distance(a: [f64; 2], b: [f64; 2]) -> f64 {
    (b[0] - a[0]).hypot(b[1] - a[1])
}

/// Parameter of the foot of the perpendicular from `point` on the line through `a` and `b`:
/// 0 at `a`, 1 at `b`.
fn foot_parameter(point: [f64; 2], a: [f64; 2], b: [f64; 2]) -> f64 {
    let (dx, dy) = (b[0] - a[0], b[1] - a[1]);
    ((point[0] - a[0]) * dx + (point[1] - a[1]) * dy) / (dx * dx + dy * dy)
}

fn distance_to_segment(point: [f64; 2], a: [f64; 2], b: [f64; 2]) -> f64 {
    if a == b {
        return distance(point, a);
    }
    let t = foot_parameter(point, a, b).clamp(0.0, 1.0);
    distance(point, [a[0] + t * (b[0] - a[0]), a[1] + t * (b[1] - a[1])])
}

// ---------------------------------------------------------------------------------------------
// Pairs of lines
// ---------------------------------------------------------------------------------------------

/// A prediction and a ground truth that have something to do with each other, and a buffer of a
/// size at which the result is neither 0 nor 1 for most pairs. `case` cycles through four kinds:
///
/// 0. gentle curve, prediction = ground truth with lateral noise
/// 1. the same with sharp turns, loops and self-crossings
/// 2. two independent gentle curves through the same small area
/// 3. two independent tangles through the same small area
pub fn random_pair(rng: &mut Rng, case: usize) -> (Line, Line, f64) {
    let buffer = rng.uniform(0.05, 0.6);
    let vertices = 3 + rng.index(18);
    let max_turn = if case % 2 == 0 {
        0.3
    } else {
        std::f64::consts::PI
    };
    let ground_truth = random_polyline(rng, vertices, max_turn);
    let prediction = if case % 4 < 2 {
        let noise = rng.uniform(0.5, 2.0) * buffer;
        ground_truth
            .iter()
            .map(|p| {
                [
                    p[0] + rng.uniform(-noise, noise),
                    p[1] + rng.uniform(-noise, noise),
                ]
            })
            .collect()
    } else {
        let vertices = 3 + rng.index(18);
        let other = random_polyline(rng, vertices, max_turn);
        // Start the second line near the middle of the first so that they meet.
        let middle = ground_truth[ground_truth.len() / 2];
        let (dx, dy) = (middle[0] - other[0][0], middle[1] - other[0][1]);
        translated(
            &other,
            dx + rng.uniform(-2.0, 2.0),
            dy + rng.uniform(-2.0, 2.0),
        )
    };
    (prediction, ground_truth, buffer)
}
