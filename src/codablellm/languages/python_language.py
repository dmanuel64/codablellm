from pathlib import Path
from typing import Final, Set

try:
    import tree_sitter_python as tsp
except ModuleNotFoundError:
    tsp = None

from tree_sitter import Language

from codablellm.core.utils import PathLike, requires_extra
from codablellm.languages.common import TreeSitterExtractor, rglob_file_extensions

TREE_SITTER_QUERY: Final[str] = (
    # Top-level function definitions
    "(function_definition"
    "  name: (identifier) @function.name) @function.definition"
    # Method definitions inside classes
    "(class_definition"
    "  name: (identifier) @class.name"
    "  body: (block"
    "    (function_definition"
    "      name: (identifier) @function.name) @function.definition))"
)
"""
Tree-sitter query for extracting function names and definitions.
"""


class PythonExtractor(TreeSitterExtractor):
    """
    Source code extractor for extracting Python functions.
    """

    def __init__(self) -> None:
        super().__init__("Python", TREE_SITTER_QUERY)  # type: ignore

    def get_extractable_files(self, path: PathLike) -> Set[Path]:
        return rglob_file_extensions(path, [".py"])

    @requires_extra("python", "Python source code extraction", "tree_sitter_python")
    def get_language(self) -> Language:
        return Language(tsp.language())  # type: ignore

    @staticmethod
    def is_installed() -> bool:
        return tsp is not None
