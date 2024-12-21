
import itertools
from pathlib import Path
from typing import Final, Sequence

from tree_sitter import Language, Parser
import tree_sitter_c as tsc

from codablellm.core.extractor import Extractor
from codablellm.core.function import SourceFunction
from codablellm.core.utils import PathLike

TREE_SITTER_QUERY: Final[str] = (
    '(function_definition'
    '   declarator: (function_declarator'
    '       declarator: (identifier) @function.name'
    '   )'
    ') @function.definition')


class CExtractor(Extractor):

    NAME: Final[str] = 'C'
    LANGUAGE: Final[Language] = Language(tsc.language())
    PARSER: Final[Parser] = Parser(LANGUAGE)

    def extract(self, path: PathLike) -> Sequence[SourceFunction]:
        functions = []
        path = Path(path)
        ast = CExtractor.PARSER.parse(path.read_bytes())
        for _, group in CExtractor.LANGUAGE.query(TREE_SITTER_QUERY).matches(ast.root_node):
            function_definition, = group['function.definition']
            function_name, = group['function.name']
            if not function_definition.text or function_name.text:
                raise ValueError('Expected function.name and function.definition to have '
                                 'text')
            functions.append(SourceFunction.from_source(path, CExtractor.NAME,
                                                        function_definition.text.decode(),
                                                        function_definition.text.decode(),
                                                        function_definition.start_byte,
                                                        function_definition.end_byte))
        return functions

    def get_extractable_files(self, path: PathLike) -> Sequence[Path]:
        path = Path(path)
        extensions = ['.c', '.h']
        if any(path.suffix.casefold() == e.casefold() for e in extensions):
            return [path]
        return list(itertools.chain.from_iterable([path.rglob(f'*{e}', case_sensitive=False)
                                                   for e in extensions]))
