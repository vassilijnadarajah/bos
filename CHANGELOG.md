# Changelog

## Unreleased

## 0.1.0 - 2026-11-16

First release.

- `calc_completeness` (recall) and `calc_correctness` (precision) for two 2D polylines, after the
  buffer overlay statistics of Tveite and Langaas (1999).
- The buffer of a line is everything within the buffer distance of it: round at bends and
  reaching past the two ends, as a GIS buffer does. The matched lengths are computed exactly from
  this definition; no buffer polygon is built.
- Flat buffer ends as an option, for a line that should get no credit beyond its own ends:
  `end_cap="flat"` in Python, `EndCap::Flat` with `calc_completeness_with_end_cap` and
  `calc_correctness_with_end_cap` in Rust.
- Input without a result is an error, never a number: a line with fewer than two distinct points,
  a coordinate that is NaN or infinite, a buffer that is not a finite number greater than 0.
  Python raises `ValueError`; Rust returns `MetricError`.
- Python: the module `bos_geo` for CPython 3.9 and later. A line is a sequence of `(x, y)` pairs or
  an `(N, 2)` NumPy array. A type stub and a `py.typed` marker are included.
- Rust: the crate has no dependencies. The Python binding is behind the cargo feature `python`.
