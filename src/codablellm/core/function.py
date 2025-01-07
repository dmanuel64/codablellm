
from dataclasses import dataclass
from pathlib import Path
from typing import Dict, Final, Optional, TypedDict
import uuid

from tree_sitter import Node, Parser
from tree_sitter import Language, Parser
import tree_sitter_c as tsc

from codablellm.core.utils import ASTEditor, SupportsJSON


@dataclass(frozen=True)
class Function:
    uid: str
    path: Path

    @staticmethod
    def create_uid(path: Path) -> str:
        return path.parts[-1]


@dataclass(frozen=True)
class SourceFunction(Function):
    language: str
    definition: str
    name: str
    start_byte: int
    end_byte: int
    class_name: Optional[str] = None

    def __post_init__(self) -> None:
        if self.start_byte < 0:
            raise ValueError('Start byte must be a non-negative integer')
        if self.start_byte > self.end_byte:
            raise ValueError('Start byte must be less than end byte')

    @property
    def is_method(self) -> bool:
        return self.class_name is not None

    def with_definition(self, definition: str, name: Optional[str] = None,
                        write_back: bool = True) -> 'SourceFunction':
        if not name:
            name = self.name
        uid = SourceFunction.create_uid(self.path, name,
                                        class_name=self.class_name)
        source_function = SourceFunction(uid, self.path, self.language, definition, name,
                                         self.start_byte, self.start_byte + len(definition))
        if write_back:
            # TODO: swap this out with ASTEditor
            source_code = source_function.path.read_text()
            source_function.path.write_text(source_code[:self.start_byte] +
                                            source_function.definition +
                                            source_code[self.end_byte:])
        return source_function

    @staticmethod
    def create_uid(path: Path, name: str, class_name: Optional[str] = None) -> str:
        if class_name:
            uid = f'{Function.create_uid(path)}::{class_name}.{name}'
        else:
            uid = f'{Function.create_uid(path)}::{name}'
        return uid

    @classmethod
    def from_source(cls, path: Path, language: str, definition: str, name: str, start_byte: int,
                    end_byte: int, class_name: Optional[str] = None) -> 'SourceFunction':
        return cls(SourceFunction.create_uid(path, name, class_name=class_name), path, language,
                   definition, name, start_byte, end_byte, class_name=class_name)


class DecompiledFunctionJSONObject(TypedDict):
    uid: str
    path: str
    definition: str
    name: str
    assembly: str
    architecture: str


GET_C_SYMBOLS_QUERY: Final[str] = (
    '(function_definition'
    '    declarator: (function_declarator'
    '        declarator: (identifier) @function.symbols'
    '    )'
    ')'
    '(call_expression'
    '    function: (identifier) @function.symbols'
    ')'
)
C_PARSER: Final[Parser] = Parser(Language(tsc.language()))


@dataclass(frozen=True)
class DecompiledFunction(Function, SupportsJSON):
    definition: str
    name: str
    assembly: str
    architecture: str

    def to_stripped(self) -> 'DecompiledFunction':
        definition = self.definition
        assembly = self.assembly
        symbol_mapping: Dict[str, str] = {}

        def strip(node: Node) -> str:
            nonlocal symbol_mapping, assembly
            if not node.text:
                raise ValueError('Expected all function.symbols to have '
                                 f'text: {node}')
            orig_function = node.text.decode()
            stripped_symbol = symbol_mapping.setdefault(orig_function,
                                                        f'sub_{str(uuid.uuid4()).split('-', maxsplit=1)[0]}')
            assembly = assembly.replace(orig_function, stripped_symbol)
            return stripped_symbol

        editor = ASTEditor(C_PARSER, definition)
        editor.match_and_edit(GET_C_SYMBOLS_QUERY,
                              {'function.symbols': strip})
        definition = editor.source_code
        first_function, *_ = (f for f in symbol_mapping.values()
                              if f.startswith('sub_'))
        return DecompiledFunction(self.uid, self.path, definition, first_function, assembly,
                                  self.architecture)

    def to_json(self) -> DecompiledFunctionJSONObject:
        return {'uid': self.uid, 'path': str(self.path), 'definition': self.definition,
                'name': self.name, 'assembly': self.assembly, 'architecture': self.architecture}

    @classmethod
    def from_json(cls, json_obj: DecompiledFunctionJSONObject) -> 'DecompiledFunction':
        return cls(json_obj['uid'], Path(json_obj['path']), json_obj['definition'],
                   json_obj['name'], json_obj['assembly'], json_obj['architecture'])
