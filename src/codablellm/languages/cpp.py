from pathlib import Path
from typing import Final, Set

import tree_sitter_cpp as tscpp
from tree_sitter import Language

from codablellm.core.utils import PathLike
from codablellm.languages.common import TreeSitterExtractor, rglob_file_extensions

TREE_SITTER_QUERY: Final[str] = (
    # Free-standing function definitions
    "(function_definition"
    "  declarator: (function_declarator"
    "    declarator: (identifier) @function.name)"
    ") @function.definition"
    # Out-of-class method definitions (e.g., MyClass::method)
    "(function_definition"
    "  declarator: (function_declarator"
    "    declarator: (qualified_identifier"
    "      scope: (namespace_identifier) @class.name"
    "      name: (identifier) @function.name))"
    ") @function.definition"
    # In-class method definitions
    "(class_specifier"
    "  name: (type_identifier) @class.name"
    "  body: (field_declaration_list"
    "    (function_definition"
    "      declarator: (function_declarator"
    "        declarator: (field_identifier) @function.name)"
    "    ) @function.definition))"
)
"""
Tree-sitter query for extracting function names and definitions.
"""


class CPPExtractor(TreeSitterExtractor):
    """
    Source code extractor for extracting C++ functions.
    """

    def __init__(self) -> None:
        super().__init__("C++", TREE_SITTER_QUERY)

    def get_extractable_files(self, path: PathLike) -> Set[Path]:
        return rglob_file_extensions(path, [".cpp", ".cc", ".cxx", ".c++"])

    def get_language(self) -> Language:
        return Language(tscpp.language())

    @classmethod
    def language(cls) -> str:
        return "C++"
