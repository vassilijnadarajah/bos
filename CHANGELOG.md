# Changelog

## Unreleased

### Results change

Both metrics are now computed from the definition of the buffer as everything within the buffer
distance of a line. Before, the buffer was an approximate polygon with flat ends and mitred
corners. Values differ from earlier ones in three ways:

- **Line ends.** The buffer now reaches the buffer distance past the two ends of a line, as in the
  cited method and in a GIS buffer. Results rise where one line continues past an end of the other.
  For a prediction from (0, 0) to (2, 2), a ground truth from (1, 1) to (3, 3) and a buffer of
  0.1, completeness and correctness go from 0.5 to 0.535. Pass `end_cap="flat"` (Python) or use
  `calc_completeness_with_end_cap` and `calc_correctness_with_end_cap` with `EndCap::Flat` (Rust)
  to keep ends that stop at the line.
- **Bends.** The buffer is round at a bend instead of running out into a point. At a sharp turn
  the old buffer reached many times the buffer distance and matched lines that were far away.
- **Wrong results that are now right.** These inputs used to give a value that did not follow from
  the method:
  - a line compared with itself, or with a copy shifted sideways by less than the buffer: 0 instead
    of 1 for about half of all lines, depending on rounding
  - a line with a vertex exactly on the edge of the other line's buffer
  - a point listed twice in a row
  - a line that doubles back on itself or forms a closed ring

On smooth lines, compared away from their ends, the values agree with the earlier ones to four
decimals.

### Changed

- The Python module is importable as `bos_geo`. The built module could not be imported before.
- Input without a result is an error instead of `0.0`, `NaN` or a panic: a line with fewer than
  two distinct points, a coordinate that is NaN or infinite, a buffer that is zero, negative, NaN
  or infinite. Python raises `ValueError`. A negative buffer used to count as its absolute value.
- Rust: the metric functions return `Result<f64, MetricError>` and take slices (`&[T]`) instead of
  `&Vec<T>`.
- Rust: `Point2D::abs` is now `Point2D::length`, `Point2D::norm` is now `Point2D::normalized`, and
  the dot product is `Point2D::dot` instead of `Point2D * Point2D`.
- Rust: PyO3 is an optional dependency behind the cargo feature `python`. Without it the crate has
  no dependencies, and `cargo test` needs no Python.

### Added

- The choice of buffer ends: `end_cap` in Python, `EndCap` and the `*_with_end_cap` functions in
  Rust.
- `MetricError` and `LineRole`, and `Point2D::cross`.
- Documentation of every public item in Rust and of the Python functions, with tested examples.
- A type stub with a `py.typed` marker in the Python package.
- Tests against a brute-force reference and of the properties of the metrics, Python tests, a
  benchmark (`cargo bench`) and continuous integration.

### Removed

- Rust: `create_line_buffer`, `calc_line_polygon_overlay` and `calc_line_length`, and with them
  public access to the `geometry` module. The metrics no longer build a polygon.

### Performance

- One call is about three times faster than before on lines of 1 000 to 20 000 vertices.
