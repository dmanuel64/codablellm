import inspect
import json
from pathlib import Path
import platform
from typing import Any, Callable, Dict, List, Optional, Sequence, Tuple, Type, Union

import pytest

from codablellm.core.decompiler import Decompiler
from codablellm.core.function import DecompiledFunction, SourceFunction
from codablellm.core.utils import DynamicSymbol, PathLike


@pytest.fixture
def create_function_factory() -> Callable[
    [Path, str, str, str, Optional[int], Optional[int], Optional[str]],
    SourceFunction,
]:
    def create_function(
        source_path: Path,
        language: str,
        definition: str,
        name: str,
        start_byte: Optional[int] = None,
        end_byte: Optional[int] = None,
        class_name: Optional[str] = None,
    ) -> SourceFunction:
        if start_byte is None:
            start_byte = 0
        if end_byte is None:
            end_byte = len(definition.encode())
        # Write to source code file
        if source_path.exists():
            source_code = source_path.read_text()
        else:
            source_code = ""
        before_definition = source_code[:start_byte]
        after_definition = source_code[start_byte:]
        new_source_code = before_definition + definition + after_definition
        source_path.write_text(new_source_code)
        return SourceFunction.from_source(
            source_path,
            language,
            definition,
            name,
            start_byte=start_byte,
            end_byte=end_byte,
            class_name=class_name,
        )

    return create_function


@pytest.fixture
def create_compiled_functions_factory(create_function_factory) -> Callable[
    [Path, Path, str, str, str, Optional[int], Optional[int], Optional[str]],
    Tuple[SourceFunction, DecompiledFunction],
]:
    def create_compiled_functions(
        source_path: Path,
        bin_path: Path,
        language: str,
        definition: str,
        name: str,
        start_byte: Optional[int] = None,
        end_byte: Optional[int] = None,
        class_name: Optional[str] = None,
    ) -> Tuple[SourceFunction, DecompiledFunction]:
        nonlocal create_function_factory
        source_function = create_function_factory(
            source_path,
            language,
            definition,
            name,
            start_byte=start_byte,
            end_byte=end_byte,
            class_name=class_name,
        )
        # Write to mock binary
        decompiled_function = DecompiledFunction.from_decompiled_json(
            {
                "path": str(bin_path),
                "name": name,
                "definition": definition,
                "assembly": "...",
                "architecture": platform.machine(),
                "address": 0x1000,
            }
        )
        bin_path.touch()
        with open(bin_path, "r+") as bin_file:
            try:
                bin_funcs: List[Dict[str, Any]] = json.load(bin_file)
            except json.JSONDecodeError:
                bin_funcs = []
            bin_funcs.append(decompiled_function.to_json())  # type: ignore
            json.dump(bin_funcs, bin_file)
        return (
            source_function,
            decompiled_function,
        )

    return create_compiled_functions


class MockDecompiler(Decompiler):
    def decompile(self, path: PathLike) -> Sequence[DecompiledFunction]:
        with open(path, "r") as bin_file:
            bin_funcs: List[Dict[str, Any]] = json.load(bin_file)
            return [
                DecompiledFunction.from_json(bin_func) for bin_func in bin_funcs  # type: ignore
            ]

    def get_stripped_function_name(self, address: int) -> str:
        return f"FUN_{address:X}"


@pytest.fixture()
def mock_decompiler() -> DynamicSymbol:
    """
    Provides a mock decompiler class for testing
    """

    return DynamicSymbol.from_str(f"{__file__}::{MockDecompiler.__name__}")
