from pathlib import Path
import platform
from typing import Callable, Optional, Sequence, Tuple

import pytest

from codablellm.core.decompiler import Decompiler
from codablellm.core.function import DecompiledFunction, SourceFunction
from codablellm.core.utils import DynamicSymbol, PathLike


@pytest.fixture
def dummy_decompiled_function(tmp_path: Path) -> DecompiledFunction:
    """
    Provides a reusable mock `DecompiledFunction` instance used across multiple tests.
    """
    return DecompiledFunction(
        uid="test",
        path=tmp_path.with_name("test.exe"),
        name="test_function",
        definition="int test() { return 0; }",
        assembly="test: mov eax, 0",
        architecture="x86_64",
        address=0x400080,
    )


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
                "definition": "...",
                "assembly": "...",
                "architecture": platform.machine(),
                "address": 0x1000,
            }
        )
        with open(bin_path, "a") as bin_file:
            bin_file.write(str(decompiled_function.to_json()))
        return (
            source_function,
            decompiled_function,
        )

    return create_compiled_functions


@pytest.fixture
def mock_decompiler(
    dummy_decompiled_function: DecompiledFunction,
) -> Decompiler:
    """
    Provides a mock decompiler class for testing
    """

    class MockDecompiler(Decompiler):
        def decompile(self, path: PathLike) -> Sequence[DecompiledFunction]:
            return [dummy_decompiled_function]

        def get_stripped_function_name(self, address: int) -> str:
            return f"FUN_{address:X}"

    return MockDecompiler()


@pytest.fixture
def dummy_c_file(tmp_path: Path) -> Path:
    """
    Provides a reusable C source code file used across multiple tests.
    """
    tmp_path = tmp_path / "test.c"
    tmp_path.write_text("int test() { return 0; }")
    return tmp_path


@pytest.fixture
def dummy_transform_symbol(tmp_path: Path) -> DynamicSymbol:
    """
    Provides a reusable Python file with a transform used across multiple tests.
    """
    tmp_path = tmp_path / "transform.py"
    tmp_path.write_text(
        (
            "def dummy_transform(sf):"
            "return sf.with_definition("
            '    "int transformed_function(int arg) { return 1; }",'
            '    name="transformed_function",'
            ")"
        )
    )
    return (tmp_path, "dummy_transform")
