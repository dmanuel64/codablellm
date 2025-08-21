from pathlib import Path
from typing import Final, List, Tuple

import pytest

from codablellm.core import *


# TODO: improve testing speed by setting up scopes for reused fixtures
@pytest.fixture()
def mock_c_function(
    tmp_path, create_compiled_functions_factory
) -> Tuple[SourceFunction, DecompiledFunction]:
    c_function_definition = (
        "\n" "\nint main(int argc, char **argv) {" "\n    return 0;" "\n}" "\n"
    )
    source_file = tmp_path / "test.c"
    bin_file = source_file.with_suffix(".exe")
    return create_compiled_functions_factory(
        source_file, bin_file, "C", c_function_definition, "main"
    )


def test_set_and_get_decompiler(mock_decompiler):
    """
    Ensures `decompiler.set()` and `decompiler.get()` functions correctly register and return the active decompiler.
    """
    decompiler.set(mock_decompiler)
    symbol = decompiler.get()
    assert symbol.symbol == "MockDecompiler"
    assert symbol.path == Path(__file__).parent.parent / "conftest.py"


def test_pseudo_strip(mock_decompiler, mock_c_function):
    """
    Validates that `pseudo_strip()` replaces original function symbols with anonymized placeholders.
    """
    _, bin_function = mock_c_function
    decompiler.set(mock_decompiler)
    decompiler_instance = decompiler.create_decompiler()
    stripped = decompiler.pseudo_strip(decompiler_instance, bin_function)
    assert bin_function.name not in stripped.definition
    assert f"FUN_{bin_function.address:x}" in stripped.definition


def test_decompile_task(
    mock_decompiler,
    mock_c_function,
):
    """
    Checks if `decompile_task` correctly calls `decompile_stripped()` when a symbol remover is specified.
    """

    _, bin_function = mock_c_function
    decompiler.set(mock_decompiler)
    decompiler_instance = decompiler.create_decompiler()

    result: List[DecompiledFunction] = decompiler.decompile_task.fn(
        decompiler_instance, bin_function.path, "pseudo-strip"
    )
    assert len(result) == 1


def test_decompile(
    mock_decompiler,
    mock_c_function,
):
    """ "
    Tests the high-level `decompile` function's return decompiled functions.
    """

    config = DecompileConfig(recursive=True)
    decompiler.set(mock_decompiler)
    results = decompiler.decompile(mock_c_function[1].path, config=config, as_flow=False)
    assert isinstance(results, list)
    assert results[0].name == "test_function"
