from pathlib import Path
from typing import Final, Set

try:
    import tree_sitter_javascript as tsjs
except ModuleNotFoundError:
    tsjs = None

from tree_sitter import Language

from codablellm.core.utils import PathLike, requires_extra
from codablellm.languages.common import TreeSitterExtractor, rglob_file_extensions

TREE_SITTER_QUERY: Final[str] = (
    # Function declarations
    "(function_declaration"
    "  name: (identifier) @function.name) @function.definition"
    # Function expressions (e.g., const foo = function(...) {...};)
    "(variable_declarator"
    "  name: (identifier) @function.name"
    "  value: (function_expression)) @function.definition"
    # Arrow functions (e.g., const foo = (...) => {...};)
    "(variable_declarator"
    "  name: (identifier) @function.name"
    "  value: (arrow_function)) @function.definition"
    # Method definitions in classes
    "(class_declaration"
    "  name: (identifier) @class.name"
    "  body: (class_body"
    "    (method_definition"
    "      name: (property_identifier) @function.name) @function.definition))"
    # Method definitions in class expressions (e.g., const Foo = class {...})
    "(variable_declarator"
    "  name: (identifier) @class.name"
    "  value: (class"
    "    body: (class_body"
    "      (method_definition"
    "        name: (property_identifier) @function.name) @function.definition)))"
)
"""
Tree-sitter query for extracting function names and definitions.
"""


class JavaScriptExtractor(TreeSitterExtractor):
    """
    Source code extractor for extracting JavaScript functions.
    """

    def __init__(self) -> None:
        super().__init__("JavaScript", TREE_SITTER_QUERY)

    def get_extractable_files(self, path: PathLike) -> Set[Path]:
        return rglob_file_extensions(path, [".js", ".cjs", ".mjs"])

    @requires_extra(
        "javascript", "JavaScript source code extraction", "tree_sitter_javascript"
    )
    def get_language(self) -> Language:
        return Language(tsjs.language())  # type: ignore

    @staticmethod
    def is_installed() -> bool:
        return tsjs is not None
