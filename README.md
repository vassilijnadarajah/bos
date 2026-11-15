# Buffer Overlay Statistics

`bos-geo` scores a predicted 2D line against a ground truth line. It answers two questions:

- **Completeness** (recall): how much of the ground truth was found?
- **Correctness** (precision): how much of the prediction is right?

These are the quantitative metrics commonly used in automatic road alignment and curb detection.
The library works for any two polylines in planar coordinates. It is written in Rust without
dependencies, and the same code is available as a Python module.

| | Name |
|---|---|
| Python distribution | `bos-geo` |
| Python module | `bos_geo` |
| Rust crate | `bos-geo` (library `bos_geo`) |

## Background

The method is the buffer overlay statistics (BOS) of Tveite and Langaas (1999). A part of one line
counts as matched if it lies within a tolerance of the other line. The zone within that tolerance
of a line is the line's *buffer*.

```text
completeness = Length(ground truth inside the buffer of the prediction) / Length(ground truth)
correctness  = Length(prediction inside the buffer of the ground truth) / Length(prediction)
```

Both are shares between 0 and 1. The paper reports `1 - correctness` under the name *miscodings*.

### The buffer

The buffer of a line is every point that is at most the distance `buffer` away from it. It is
`2 · buffer` wide, round where the line bends, and it reaches `buffer` past the two ends of the
line. This is the buffer that a GIS buffer operation produces.

If a line should not be credited for anything beyond its own ends, choose flat ends
(`end_cap="flat"` in Python, `EndCap::Flat` in Rust). Bends stay round.

```text
        round ends (default)                 flat ends

     .---------------------.            +-----------------+
    (   o-----------------o   )         o-----------------o
     '---------------------'            +-----------------+
```

The lengths are computed exactly from this definition. No buffer polygon is built: the buffer is
the union of a rectangle around every segment and a disc around every vertex, a segment of the
other line meets each of these in one interval that has a closed form, and the matched part is the
union of the intervals.

### What the method does not see

Only distance counts.

- **Direction.** A prediction crossing the ground truth at a right angle is credited with the
  stretch that lies inside the buffer.
- **Shape.** A prediction that wiggles inside the buffer scores like a straight one.
- **Redundancy.** One call compares one line with one line. Sets of lines are not supported.

## Conventions

- Arguments are always `(prediction, ground_truth, buffer)`.
- A line is its points in order. A point repeated in a row is ignored.
- Coordinates are planar. Both lines and `buffer` must be in the same unit and the same projected
  coordinate system, for example UTM. Longitude and latitude in degrees give meaningless results.
- `buffer` is the half-width of the buffer, not its full width.
- Input without a result is an error, never a number: a line with fewer than two distinct points,
  a coordinate that is NaN or infinite, or a `buffer` that is not a finite number greater than 0.

## Installation

### Python

```sh
pip install bos-geo    # from PyPI, once a release is published there
pip install .          # from a checkout of this repository; needs a Rust toolchain
```

CPython 3.8 to 3.13 is supported. NumPy is optional.

### Rust

```toml
[dependencies]
bos-geo = { git = "https://github.com/vassilijnadarajah/bos" }
```

The Python binding is behind the cargo feature `python`, which is off by default: the crate builds
without Python and without any dependency.

## Usage

The examples use a prediction that covers the first half of the ground truth. Four of the eight
units of ground truth lie beside it, and the round end of its buffer reaches another 0.5.

### Python

A line is a sequence of `(x, y)` pairs or an `(N, 2)` NumPy array.

```python
>>> import bos_geo
>>> prediction = [(0, 0), (4, 0)]
>>> ground_truth = [(0, 0), (8, 0)]

>>> bos_geo.calc_completeness(prediction, ground_truth, buffer=0.5)
0.5625
>>> bos_geo.calc_correctness(prediction, ground_truth, buffer=0.5)
1.0
>>> bos_geo.calc_completeness(prediction, ground_truth, buffer=0.5, end_cap="flat")
0.5

>>> import numpy as np
>>> bos_geo.calc_completeness(np.array(prediction), np.array(ground_truth), buffer=0.5)
0.5625
```

Unusable input raises `ValueError`, and the message names the argument:

```python
>>> bos_geo.calc_completeness(prediction, [(1, 1)], buffer=0.5)
Traceback (most recent call last):
    ...
ValueError: ground_truth has 1 point(s), a line needs at least 2
```

### Rust

A line is a slice of `Point2D`, `[x, y]` or `(x, y)`.

```rust
use bos_geo::{EndCap, MetricError, calc_completeness, calc_completeness_with_end_cap};
use bos_geo::{LineRole, calc_correctness};

fn main() -> Result<(), MetricError> {
    let prediction = [[0.0, 0.0], [4.0, 0.0]];
    let ground_truth = [[0.0, 0.0], [8.0, 0.0]];

    assert_eq!(calc_completeness(&prediction, &ground_truth, 0.5)?, 0.5625);
    assert_eq!(calc_correctness(&prediction, &ground_truth, 0.5)?, 1.0);

    let flat = calc_completeness_with_end_cap(&prediction, &ground_truth, 0.5, EndCap::Flat)?;
    assert_eq!(flat, 0.5);

    // Unusable input is an error that names the argument.
    let error = calc_completeness(&prediction, &[[1.0, 1.0]], 0.5).unwrap_err();
    assert_eq!(error, MetricError::TooFewPoints { line: LineRole::GroundTruth, points: 1 });
    Ok(())
}
```

## Development

```sh
cargo test                      # unit, integration and documentation tests; needs no Python
cargo bench                     # run time of both metrics on lines of 1 000 to 20 000 vertices
cargo fmt --check
cargo clippy --all-targets --features python -- -D warnings

pip install maturin pytest numpy
maturin develop                 # build the Python module into the active virtual environment
pytest tests/python
```

The tests in `tests/` compare the metrics with a brute-force reference on seeded random lines and
check the properties that follow from the definition. `tests/python` covers what the binding adds.
The Python docstrings live in `src/python.rs` and are repeated in the type stub `bos_geo.pyi`; a
test fails if the two differ.

## Reference

Tveite, H. and Langaas, S. (1999). An accuracy assessment method for geographical line data sets
based on buffering. *International Journal of Geographical Information Science*, 13(1), 27–47.

## License

This project is licensed under the GPLv3 license.
