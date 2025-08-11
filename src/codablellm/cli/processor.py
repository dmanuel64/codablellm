from pathlib import Path
from typing import Any, Type

from typer import BadParameter, Exit

import codablellm
from codablellm.core.extractor import Extractor
from codablellm.core.utils import DynamicSymbol
from codablellm.decompilers.ghidra import Ghidra
from codablellm.languages.c import CExtractor
from codablellm.languages.cpp import CPPExtractor
from codablellm.languages.java import JavaExtractor
from codablellm.languages.javascript import JavaScriptExtractor
from codablellm.languages.python_language import PythonExtractor
from codablellm.languages.rust import RustExtractor
from codablellm.languages.typescript import TypeScriptExtractor


def validate_dataset_format(path: Path) -> Path:
    if path.suffix.casefold() not in [
        e.casefold()
        for e in [
            ".json",
            ".jsonl",
            ".csv",
            ".tsv",
            ".xlsx",
            ".xls",
            ".xlsm",
            ".md",
            ".markdown",
            ".tex",
            ".html",
            ".html",
            ".xml",
        ]
    ]:
        raise BadParameter(f'Unsupported dataset format: "{path.suffix}"')
    return path


def parse_symbol(value: Any) -> DynamicSymbol:
    value = str(value)
    # Check for common classes/functions for decompilers/extractors/mappers
    def verify_language_support(extractor_type: Type[Extractor], extra: str) -> None:
        if not extractor_type.is_installed():
            raise BadParameter(
                f'{extractor_type.language()} language support requires the "{extra}" extra to be installed. '
                f'Install with "pip install codablellm[{extra}]" or "pip install codablellm[langs]" to '
                "install support for all languages."
            )

    value = value.lower()
    if value == "ghidra":
        symbol = DynamicSymbol.from_builtin_symbol(Ghidra)
    elif value == "c":
        symbol = DynamicSymbol.from_builtin_symbol(CExtractor)
    elif value == "c++":
        symbol = DynamicSymbol.from_builtin_symbol(CPPExtractor)
    elif value == "java":
        symbol = DynamicSymbol.from_builtin_symbol(JavaExtractor)
        verify_language_support(JavaExtractor, "java")
    elif value == "javascript":
        symbol = DynamicSymbol.from_builtin_symbol(JavaScriptExtractor)
        verify_language_support(JavaExtractor, "javascript")
    elif value == "python":
        symbol = DynamicSymbol.from_builtin_symbol(PythonExtractor)
        verify_language_support(JavaExtractor, "python")
    elif value == "rust":
        symbol = DynamicSymbol.from_builtin_symbol(RustExtractor)
        verify_language_support(JavaExtractor, "rust")
    elif value == "typescript":
        symbol = DynamicSymbol.from_builtin_symbol(TypeScriptExtractor)
        verify_language_support(JavaExtractor, "typescript")
    else:
        try:
            symbol = DynamicSymbol.from_str(value)
        except ValueError as e:
            raise BadParameter(
                "Class/function must be in the format of 'path/to/file.py::ClassOrFunction'"
            ) from e
    return symbol

def show_version(show: bool) -> None:
    if show:
        print(f"[b]codablellm {codablellm.__version__}")
        raise Exit()