# SPDX-License-Identifier: GPL-3.0-only
# (C) 2025 Vassilij Nadarajah
# packages@vassilijnadarajah.de

"""Tests of the Python module. The geometry itself is tested in Rust; these cover what the
binding adds: the accepted input types, the keyword argument, the errors, and the type stub.

Build and install the module first, for example with `maturin develop`.
"""

import ast
import doctest
import inspect
import math
from importlib import resources

import numpy as np
import pytest

import bos_geo

METRICS = [bos_geo.calc_completeness, bos_geo.calc_correctness]

# The prediction runs 0.3 beside the first part of a ground truth that is twice as long. The two
# lines are side by side for 3 units; a round buffer end of 0.5 reaches sqrt(0.5² - 0.3²) = 0.4
# further.
PREDICTION = [(0.0, 0.0), (4.0, 0.0)]
GROUND_TRUTH = [(1.0, 0.3), (9.0, 0.3)]
BUFFER = 0.5


def test_completeness():
    assert bos_geo.calc_completeness(PREDICTION, GROUND_TRUTH, BUFFER) == pytest.approx(3.4 / 8)


def test_correctness():
    assert bos_geo.calc_correctness(PREDICTION, GROUND_TRUTH, BUFFER) == pytest.approx(3.4 / 4)


def test_flat_ends():
    completeness = bos_geo.calc_completeness(PREDICTION, GROUND_TRUTH, BUFFER, end_cap="flat")
    correctness = bos_geo.calc_correctness(PREDICTION, GROUND_TRUTH, BUFFER, end_cap="flat")
    assert completeness == pytest.approx(3 / 8)
    assert correctness == pytest.approx(3 / 4)


def test_round_ends_are_the_default():
    for metric in METRICS:
        default = metric(PREDICTION, GROUND_TRUTH, BUFFER)
        assert metric(PREDICTION, GROUND_TRUTH, BUFFER, end_cap="round") == default


def test_arguments_by_keyword():
    value = bos_geo.calc_completeness(
        prediction=PREDICTION, ground_truth=GROUND_TRUTH, buffer=BUFFER
    )
    assert value == pytest.approx(3.4 / 8)


def test_end_cap_is_keyword_only():
    for metric in METRICS:
        with pytest.raises(TypeError):
            metric(PREDICTION, GROUND_TRUTH, BUFFER, "flat")


@pytest.mark.parametrize(
    "convert",
    [
        pytest.param(lambda line: [tuple(p) for p in line], id="list of tuples"),
        pytest.param(lambda line: [list(p) for p in line], id="list of lists"),
        pytest.param(lambda line: tuple(tuple(p) for p in line), id="tuple of tuples"),
        pytest.param(lambda line: np.array(line), id="float64 array"),
        pytest.param(lambda line: np.array(line, dtype=np.float32), id="float32 array"),
        pytest.param(lambda line: [np.array(p) for p in line], id="list of arrays"),
    ],
)
def test_accepted_line_types(convert):
    # Coordinates that float32 holds exactly, so every representation is the same line.
    prediction = [(0.0, 0.0), (4.0, 0.0)]
    ground_truth = [(1.0, 0.25), (9.0, 0.25)]
    expected = [metric(prediction, ground_truth, BUFFER) for metric in METRICS]
    actual = [metric(convert(prediction), convert(ground_truth), BUFFER) for metric in METRICS]
    assert actual == expected


def test_integer_coordinates():
    assert bos_geo.calc_completeness([(0, 0), (4, 0)], np.array([[0, 0], [8, 0]]), 1) == 0.625


def test_numpy_scalars_as_buffer():
    assert bos_geo.calc_completeness(PREDICTION, GROUND_TRUTH, np.float64(BUFFER)) == (
        bos_geo.calc_completeness(PREDICTION, GROUND_TRUTH, BUFFER)
    )


LINE = [(0.0, 0.0), (1.0, 0.5), (2.0, 0.0)]

INVALID = [
    pytest.param(LINE, [], 0.1, "ground_truth has 0 point(s)", id="ground truth empty"),
    pytest.param([], LINE, 0.1, "prediction has 0 point(s)", id="prediction empty"),
    pytest.param(LINE, np.empty((0, 2)), 0.1, "ground_truth has 0 point(s)", id="empty array"),
    pytest.param(LINE, [(1, 1)], 0.1, "ground_truth has 1 point(s)", id="ground truth one point"),
    pytest.param([(1, 1)], LINE, 0.1, "prediction has 1 point(s)", id="prediction one point"),
    pytest.param(
        LINE, [(1, 1)] * 3, 0.1, "ground_truth has no length", id="ground truth all identical"
    ),
    pytest.param(
        [(1, 1)] * 3, LINE, 0.1, "prediction has no length", id="prediction all identical"
    ),
    pytest.param(
        LINE,
        [(0, 0), (1, math.nan), (2, 0)],
        0.1,
        "ground_truth has a coordinate that is NaN or infinite at index 1",
        id="NaN in ground truth",
    ),
    pytest.param(
        [(math.inf, 0), (1, 0)],
        LINE,
        0.1,
        "prediction has a coordinate that is NaN or infinite at index 0",
        id="infinity in prediction",
    ),
    pytest.param(LINE, LINE, 0, "buffer must be a finite number greater than 0", id="buffer 0"),
    pytest.param(
        LINE, LINE, -0.1, "buffer must be a finite number greater than 0", id="buffer negative"
    ),
    pytest.param(
        LINE, LINE, math.nan, "buffer must be a finite number greater than 0", id="buffer NaN"
    ),
    pytest.param(
        LINE, LINE, math.inf, "buffer must be a finite number greater than 0", id="buffer infinite"
    ),
]


@pytest.mark.parametrize("metric", METRICS, ids=lambda metric: metric.__name__)
@pytest.mark.parametrize("prediction, ground_truth, buffer, message", INVALID)
def test_invalid_input_is_a_value_error(metric, prediction, ground_truth, buffer, message):
    for end_cap in ["round", "flat"]:
        with pytest.raises(ValueError) as error:
            metric(prediction, ground_truth, buffer, end_cap=end_cap)
        assert str(error.value).startswith(message)


@pytest.mark.parametrize("metric", METRICS, ids=lambda metric: metric.__name__)
def test_unknown_end_cap(metric):
    with pytest.raises(ValueError, match='end_cap must be "round" or "flat", got "square"'):
        metric(LINE, LINE, 0.1, end_cap="square")


@pytest.mark.parametrize("metric", METRICS, ids=lambda metric: metric.__name__)
def test_points_that_are_not_pairs(metric):
    with pytest.raises(ValueError):
        metric([(0, 0, 0), (1, 0, 0)], LINE, 0.1)
    with pytest.raises(TypeError):
        metric([("a", "b"), ("c", "d")], LINE, 0.1)
    with pytest.raises(TypeError):
        metric(None, LINE, 0.1)


# --- Documentation and type stub --------------------------------------------------------------


def stub():
    """The type stub as it is installed next to the compiled module."""
    source = resources.files("bos_geo").joinpath("__init__.pyi").read_text(encoding="utf-8")
    return ast.parse(source)


def test_package_is_typed():
    assert resources.files("bos_geo").joinpath("py.typed").is_file()


def test_stub_documents_the_module_like_the_module_itself():
    assert ast.get_docstring(stub()) == inspect.getdoc(bos_geo)


@pytest.mark.parametrize("metric", METRICS, ids=lambda metric: metric.__name__)
def test_stub_matches_the_function(metric):
    declared = {
        node.name: node for node in stub().body if isinstance(node, ast.FunctionDef)
    }
    assert set(declared) == {metric.__name__ for metric in METRICS}
    node = declared[metric.__name__]

    assert ast.get_docstring(node) == inspect.getdoc(metric)

    signature = inspect.signature(metric)
    positional = [arg.arg for arg in node.args.args]
    keyword_only = [arg.arg for arg in node.args.kwonlyargs]
    assert positional + keyword_only == list(signature.parameters)
    assert keyword_only == [
        name
        for name, parameter in signature.parameters.items()
        if parameter.kind is inspect.Parameter.KEYWORD_ONLY
    ]
    defaults = {
        name: ast.literal_eval(default)
        for name, default in zip(keyword_only, node.args.kw_defaults)
        if default is not None
    }
    assert defaults == {
        name: parameter.default
        for name, parameter in signature.parameters.items()
        if parameter.default is not inspect.Parameter.empty
    }


@pytest.mark.parametrize("documented", [bos_geo, *METRICS], ids=lambda item: item.__name__)
def test_docstring_examples(documented):
    examples = doctest.DocTestParser().get_doctest(
        inspect.getdoc(documented), {}, documented.__name__, None, 0
    )
    runner = doctest.DocTestRunner()
    runner.run(examples)
    assert runner.failures == 0
    if documented is not bos_geo:
        assert runner.tries > 0
