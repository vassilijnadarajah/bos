"""
## Buffer Overlay Statistics

This library provides functions for evaluating quantative metrics commonly used in automatic road alignment and curb detection.

### Overview

### Usage
```
import bos
```
"""

from typing import List, Tuple, Union
import numpy as np

def calc_completeness(prediction: Union[List[Tuple[float, float]], List[List[float]], np.ndarray], ground_truth: Union[List[Tuple[float, float]], List[List[float]], np.ndarray], buffer: float) -> float:
    """
    Calculates the completeness of a 2D line compared to the ground truth by using buffer-overlay-statistics.

    Parameters:
        prediction: A list of (x, y) tuples, a list of [x, y] lists, or a numpy array representing the predicted line.
        ground_truth: A list of (x, y) tuples, a list of [x, y] lists, or a numpy array representing the ground truth line.
        buffer: The buffer distance to consider for completeness.

    Returns:
        The completeness of the predicted line compared to the ground truth, represented by a value between 0 and 1.

    Example:
        ToDo
    """