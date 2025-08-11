from pathlib import Path
from typing import Final, Optional, Sequence, Set

try:
    import tree_sitter_typescript as tst
except ModuleNotFoundError:
    tsp = None

from tree_sitter import Language

from codablellm.core.function import SourceFunction
from codablellm.core.utils import PathLike, requires_extra
from codablellm.languages.common import TreeSitterExtractor, rglob_file_extensions

TREE_SITTER_QUERY: Final[str] = (
    # Top-level function declarations
    "(function_declaration"
    "  name: (identifier) @function.name) @function.definition"
    # Function expressions assigned to variables
    "(lexical_declaration"
    "  (variable_declarator"
    "    name: (identifier) @function.name"
    "    value: (function_expression) @function.definition))"
    # Arrow functions assigned to variables
    "(lexical_declaration"
    "  (variable_declarator"
    "    name: (identifier) @function.name"
    "    value: (arrow_function) @function.definition))"
    # Methods in class bodies
    "(class_declaration"
    "  name: (type_identifier) @class.name"
    "  body: (class_body"
    "    (method_definition"
    "      name: (property_identifier) @function.name)"
    "    @function.definition))"
    # Object methods using function expressions
    "(pair"
    "  key: (property_identifier) @function.name"
    "  value: (function_expression) @function.definition)"
    ""
    "(pair"
    "  key: (property_identifier) @function.name"
    "  value: (arrow_function) @function.definition)"
    ""
    "(pair"
    "  key: (string) @function.name"
    "  value: (function_expression) @function.definition)"
    ""
    "(pair"
    "  key: (string) @function.name"
    "  value: (arrow_function) @function.definition)"
)
"""
Tree-sitter query for extracting function names and definitions.
"""


class TypeScriptExtendedExtractor(TreeSitterExtractor):

    def __init__(self) -> None:
        super().__init__("TypeScript Extended", TREE_SITTER_QUERY)

    def get_extractable_files(self, path: PathLike) -> Set[Path]:
        return rglob_file_extensions(path, [".tsx"])

    @requires_extra(
        "typescript", "TypeScript source code extraction", "tree_sitter_typescript"
    )
    def get_language(self) -> Language:
        return Language(tst.language_tsx())  # type: ignore

    @staticmethod
    def is_installed() -> bool:
        return tst is not None


class TypeScriptExtractor(TreeSitterExtractor):
    """
    Source code extractor for extracting TypeScript functions.
    """

    def __init__(self, include_extended: bool = True) -> None:
        super().__init__("TypeScript", TREE_SITTER_QUERY)
        self._tsx_extractor = (
            TypeScriptExtendedExtractor() if include_extended else None
        )

    def extract(
        self, file_path: PathLike, repo_path: Optional[PathLike] = None
    ) -> Sequence[SourceFunction]:
        if self._tsx_extractor and Path(file_path).suffix == ".tsx":
            return self._tsx_extractor.extract(file_path, repo_path=repo_path)
        return super().extract(file_path, repo_path)

    def get_extractable_files(self, path: PathLike) -> Set[Path]:
        files = rglob_file_extensions(path, [".ts"])
        if self._tsx_extractor:
            files.update(self._tsx_extractor.get_extractable_files(path))
        return files

    @requires_extra(
        "typescript", "TypeScript source code extraction", "tree_sitter_typescript"
    )
    def get_language(self) -> Language:
        return Language(tst.language_typescript())  # type: ignore

    @staticmethod
    def is_installed() -> bool:
        return tst is not None
