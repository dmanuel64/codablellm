from typing import Callable, Final, Union
from codablellm.core.function import DecompiledFunction, SourceFunction
from codablellm.core.utils import DynamicSymbol


def name_mapper(function: DecompiledFunction, uid: Union[SourceFunction, str]) -> bool:
    """
    Maps a decompiled function to a source function by comparing their function names.

    Parameters:
        function: The decompiled function to map.
        uid: The source function UID or a `SourceFunction` object to map against.

    Returns:
        `True` if the decompiled function name matches the source function name.
    """
    if isinstance(uid, SourceFunction):
        uid = uid.uid
    return function.name == SourceFunction.get_function_name(uid)


def rust_linux_mapper(decompiled: DecompiledFunction, source: SourceFunction) -> bool:
    return decompiled.name == source.uid


def cpp_linux_mapper(decompiled: DecompiledFunction, source: SourceFunction) -> bool:
    try:
        *_, class_name, function_name = decompiled.name.split("::")
    except ValueError as e1:
        try:
            (function_name,) = decompiled.name.split("::")
        except ValueError as e2:
            raise ValueError(
                f"Malformed function name: {repr(decompiled.name)}"
            ) from e2
        else:
            class_name = None
    return function_name == source.name and class_name == source.class_name


def default_mapper(decompiled: DecompiledFunction, source: SourceFunction) -> bool:
    language = source.language.lower()
    if language == "rust":
        return rust_linux_mapper(decompiled, source)
    elif language == "c++":
        return cpp_linux_mapper(decompiled, source)
    return name_mapper(decompiled, source)


Mapper = Callable[[DecompiledFunction, SourceFunction], bool]
"""
Callable type describing a mapping function that determines if a decompiled function
corresponds to a given source function.
"""


DEFAULT_MAPPER: Final[DynamicSymbol] = DynamicSymbol.from_builtin_symbol(default_mapper)
"""
The default mapping function used to match decompiled functions to source functions.
"""
