from pathlib import Path
from typing import Final, Set


try:
    import tree_sitter_java as tsj
except ModuleNotFoundError:
    tsj = None

from tree_sitter import Language

from codablellm.core.utils import PathLike, requires_extra
from codablellm.languages.common import TreeSitterExtractor, rglob_file_extensions

TREE_SITTER_QUERY: Final[str] = (
    # Methods in classes
    "(class_declaration"
    "  name: (identifier) @class.name"
    "  body: (class_body"
    "    (method_declaration"
    "      name: (identifier) @function.name)"
    "    @function.definition))"
    # Constructors in classes
    "(class_declaration"
    "  name: (identifier) @class.name"
    "  body: (class_body"
    "    (constructor_declaration"
    "      name: (identifier) @function.name)"
    "    @function.definition))"
    # Methods in interfaces
    "(interface_declaration"
    "  name: (identifier) @interface.name"
    "  body: (interface_body"
    "    (method_declaration"
    "      name: (identifier) @function.name)"
    "    @function.definition))"
    # Methods in enums
    "(enum_declaration"
    "  name: (identifier) @enum.name"
    "  body: (enum_body"
    "    (enum_body_declarations"
    "      (method_declaration"
    "        name: (identifier) @function.name)"
    "      @function.definition)))"
    # Constructors in enums
    "(enum_declaration"
    "  name: (identifier) @enum.name"
    "  body: (enum_body"
    "    (enum_body_declarations"
    "      (constructor_declaration"
    "        name: (identifier) @function.name)"
    "      @function.definition)))"
    # Methods in records (Java 16+)
    "(record_declaration"
    "  name: (identifier) @record.name"
    "  body: (class_body"
    "    (method_declaration"
    "      name: (identifier) @function.name)"
    "    @function.definition))"
    # Constructors in records
    "(record_declaration"
    "  name: (identifier) @record.name"
    "  body: (class_body"
    "    (constructor_declaration"
    "      name: (identifier) @function.name)"
    "    @function.definition))"
)
"""
Tree-sitter query for extracting function names and definitions.
"""


class JavaExtractor(TreeSitterExtractor):
    """
    Source code extractor for extracting Java functions.
    """

    def __init__(self) -> None:
        super().__init__("Java", TREE_SITTER_QUERY)

    def get_extractable_files(self, path: PathLike) -> Set[Path]:
        return rglob_file_extensions(path, [".java"])

    @requires_extra("java", "Java source code extraction", "tree_sitter_java")
    def get_language(self) -> Language:
        return Language(tsj.language())  # type: ignore

    @staticmethod
    def is_installed() -> bool:
        return tsj is not None
