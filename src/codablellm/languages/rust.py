from pathlib import Path
from typing import Final, Set

try:
    import tree_sitter_rust as tsr
except ModuleNotFoundError:
    tsr = None

from tree_sitter import Language

from codablellm.core.utils import PathLike, requires_extra
from codablellm.languages.common import TreeSitterExtractor, rglob_file_extensions

TREE_SITTER_QUERY: Final[str] = (
    # Top-level function definitions
    "(function_item"
    "  name: (identifier) @function.name) @function.definition"
    # Method definitions inside impl blocks
    "(impl_item"
    "  type: (type_identifier) @class.name"
    "  body: (declaration_list"
    "    (function_item"
    "      name: (identifier) @function.name) @function.definition))"
)
"""
Tree-sitter query for extracting function names and definitions.
"""


class RustExtractor(TreeSitterExtractor):
    """
    Source code extractor for extracting JavaScript functions.
    """

    def __init__(self) -> None:
        super().__init__("Rust", TREE_SITTER_QUERY)

    def get_extractable_files(self, path: PathLike) -> Set[Path]:
        return rglob_file_extensions(path, [".rs"])

    @requires_extra("rust", "Rust source code extraction", "tree_sitter_rust")
    def get_language(self) -> Language:
        return Language(tsr.language())  # type: ignore

    @staticmethod
    def is_installed() -> bool:
        return tsr is not None
