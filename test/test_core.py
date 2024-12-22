from collections import deque
from pathlib import Path
from typing import List

import pytest

from codablellm.core import *
from codablellm.languages.c import CExtractor


def test_progress() -> None:
    with Progress('Doing some task...') as progress:
        progress.advance()
        assert progress.completed == 1
        progress.advance(errors=True, advance=2)
        assert progress.errors == 2
        progress.update(completed=0)
        assert progress.completed == 0
        progress.update(errors=0)
        assert progress.errors == 0


def add_numbers(left: int, right: int) -> int:
    return left + right


def test_process_pool_progress() -> None:
    numbers = [1, 2, 3, 4, 5, None]
    with ProcessPoolProgress(add_numbers, numbers,
                             Progress('Adding 3 to numbers...',
                                      total=len(numbers)),
                             submit_args=(3,)) as pool:
        sums = list(pool)
    for number in numbers[:-1]:
        assert number + 3 in sums
    assert pool.errors == 1


def concat_strs(left: str, right: str) -> str:
    return left + right


class CallableAdd(CallablePoolProgress[int, int, List[int]]):

    def __init__(self, numbers: List[int], add_first: int, add_second) -> None:
        super().__init__(ProcessPoolProgress(add_numbers, numbers,
                                             Progress(f'Adding {add_first} to numbers...',
                                                      total=len(numbers)),
                                             submit_args=(add_first,)))
        self.add_second = add_second

    def get_results(self) -> List[int]:
        return [n + self.add_second for n in self.pool]


class CallableConcat(CallablePoolProgress[str, str, List[str]]):

    def __init__(self, strings: List[str], concat_first: str, concat_second: str) -> None:
        super().__init__(ProcessPoolProgress(concat_strs, strings,
                                             Progress(f'Concatenating "{concat_first}" to strings...',
                                                      total=len(strings)),
                                             submit_args=(concat_first,)))
        self.concat_second = concat_second

    def get_results(self) -> List[str]:
        return [s + self.concat_second for s in self.pool]


def test_multi_progress() -> None:
    numbers = [1, 2, 3, 4, 5, None]
    strings = ['foo', 'bar', 'baz']
    add, concat = CallableAdd(
        numbers, 3, 5), CallableConcat(strings, 'bar', 'baz')
    sums, concat_results = deque(), deque()
    with ProcessPoolProgress.multi_progress((add, sums), (concat, concat_results)):
        pass
    for number in numbers[:-1]:
        assert number + 3 + 5 in sums
    for string in strings:
        assert f'{string}barbaz' in concat_results


def test_source_function(tmp_path: Path) -> None:
    c_definition = (
        '#include <stdio.h>'
        '\n'
        '\nint main(int argc, char **argv) {'
        '\n\tprintf("Hello, world!");'
        '\n\treturn 0;'
        '\n}')
    c_function = SourceFunction.from_source(tmp_path, 'C', c_definition, 'main',
                                            20, 92)
    assert not c_function.is_method
    cpp_definition = (
        '#include <iostream>'
        '\n'
        '\nclass Greeter {'
        '\npublic:'
        '\n\tvoid printHelloWorld() {'
        '\n\t\tstd::cout << "Hello, World!" << std::endl;'
        '\n\t}'
        '\n};'
        '\n'
        '\nint main() {'
        '\n\tGreeter greeter;'
        '\n\tgreeter.printHelloWorld();'
        '\n\treturn 0;'
        '\n}'
    )
    cpp_function = SourceFunction.from_source(tmp_path, 'C++', cpp_definition, 'printHelloWorld',
                                              46, 117, class_name='Greeter')
    assert cpp_function.is_method


def test_decompiled_function(tmp_path: Path) -> None:
    asm = (
        'addTwoNumbers:'
        '\n\tpush rbp'
        '\n\tmov rbp, rsp'
        '\n\tmov eax, edi'
        '\n\tadd eax, esi'
        '\n\tmov esi, eax'
        '\n\tlea rdi, [rel format]'
        '\n\txor eax, eax'
        '\n\tcall printf'
        '\n\tpop rbp'
        '\n\tret'
    )
    definition = (
        'void addTwoNumbers(int param_1, int param_2) {'
        '\n\tint param_3 = param_1 + param_2;'
        '\n\tprintf("The result is: %d", param_3);'
        '\n}'
    )
    uid = SourceFunction.from_source(
        tmp_path, 'C', definition, 'addTwoNumbers', 0, 1).uid
    decompiled_function = DecompiledFunction(uid, tmp_path, definition, 'addTwoNumbers', asm,
                                             'x86')
    stripped_function = decompiled_function.to_stripped()
    assert 'printf' not in stripped_function.definition
    assert 'addTwoNumbers' not in stripped_function.definition
    assert 'printf' not in stripped_function.assembly
    assert 'addTwoNumbers' not in stripped_function.assembly


def test_extractors_config() -> None:
    extractor.set_extractors({'C': 'codablellm.languages.c.CExtractor'})
    assert isinstance(extractor.get_extractor('C'), CExtractor)
    with pytest.raises(ValueError):
        extractor.get_extractor('nonexistant')
