"""
Buffer overlay statistics for 2D polylines.

Scores a predicted line against a ground truth line with the buffer method of Tveite and
Langaas (1999): a part of one line counts as matched if it lies within a tolerance, the
buffer, of the other line.

calc_completeness (recall): the share of the ground truth that was found.
calc_correctness (precision): the share of the prediction that is right.

A line is a sequence of (x, y) pairs or an (N, 2) NumPy array, in planar coordinates.
"""

# The docstrings below repeat the ones the compiled module carries (src/python.rs).
# tests/python/test_bos_geo.py compares the two, so change them together.

from typing import List, Literal, Sequence, Tuple, Union

import numpy as np

_Line = Union[Sequence[Tuple[float, float]], Sequence[List[float]], np.ndarray]

def calc_completeness(
    prediction: _Line,
    ground_truth: _Line,
    buffer: float,
    *,
    end_cap: Literal["round", "flat"] = "round",
) -> float:
    """
    Completeness (recall): the share of the ground truth that was found.

    Length(ground truth within the buffer of the prediction) / Length(ground truth)

    Parameters:
        prediction: The line being judged: its points in order, as a sequence of (x, y) pairs
            or an (N, 2) NumPy array.
        ground_truth: The line taken as correct, in the same form.
        buffer: The tolerance, as a distance in the unit of the coordinates. A point of the
            ground truth is found if it is at most this far from the prediction.
        end_cap: Shape of the buffer at the two ends of the prediction. "round" (the default)
            is everything within the buffer distance. "flat" cuts the buffer off at the ends:
            ground truth that continues past an end of the prediction is not found there.

    Returns:
        A value between 0 and 1.

    Raises:
        ValueError: If a line has fewer than two distinct points or a coordinate that is NaN
            or infinite, if buffer is not a finite number greater than 0, or if end_cap is
            neither "round" nor "flat".

    Coordinates are planar: both lines must be in the same projected coordinate system.

    Example:
        >>> import bos_geo
        >>> prediction = [(0, 0), (4, 0)]
        >>> ground_truth = [(0, 0), (8, 0)]
        >>> bos_geo.calc_completeness(prediction, ground_truth, 0.5)
        0.5625
        >>> bos_geo.calc_completeness(prediction, ground_truth, 0.5, end_cap="flat")
        0.5
    """

def calc_correctness(
    prediction: _Line,
    ground_truth: _Line,
    buffer: float,
    *,
    end_cap: Literal["round", "flat"] = "round",
) -> float:
    """
    Correctness (precision): the share of the prediction that is right.

    Length(prediction within the buffer of the ground truth) / Length(prediction)

    Parameters:
        prediction: The line being judged: its points in order, as a sequence of (x, y) pairs
            or an (N, 2) NumPy array.
        ground_truth: The line taken as correct, in the same form.
        buffer: The tolerance, as a distance in the unit of the coordinates. A point of the
            prediction is right if it is at most this far from the ground truth.
        end_cap: Shape of the buffer at the two ends of the ground truth. "round" (the default)
            is everything within the buffer distance. "flat" cuts the buffer off at the ends:
            a prediction that continues past an end of the ground truth is not right there.

    Returns:
        A value between 0 and 1.

    Raises:
        ValueError: If a line has fewer than two distinct points or a coordinate that is NaN
            or infinite, if buffer is not a finite number greater than 0, or if end_cap is
            neither "round" nor "flat".

    Coordinates are planar: both lines must be in the same projected coordinate system.

    Example:
        >>> import bos_geo
        >>> prediction = [(0, 0), (8, 0)]
        >>> ground_truth = [(0, 0), (4, 0)]
        >>> bos_geo.calc_correctness(prediction, ground_truth, 0.5)
        0.5625
        >>> bos_geo.calc_correctness(prediction, ground_truth, 0.5, end_cap="flat")
        0.5
    """
