"""
Module containing functions for managing and registering source code extractors.

Source code extractors are responsible for parsing and extracting function definitions from different programming languages.
"""

import logging
from abc import ABC, abstractmethod
from dataclasses import dataclass, field
from pathlib import Path
from typing import (
    Any,
    Callable,
    Dict,
    Final,
    FrozenSet,
    List,
    Mapping,
    NamedTuple,
    Optional,
    Sequence,
    Set,
    Type,
    Union,
)

from codablellm.core.function import SourceFunction
from codablellm.core.utils import (
    DynamicSymbol,
    PathLike,
    codablellm_flow,
    codablellm_low_level_task,
    codablellm_task,
    dynamic_import,
)


_EXTRACTORS: Final[Dict[str, DynamicSymbol]] = {}

logger = logging.getLogger(__name__)


def get_registered() -> Dict[str, DynamicSymbol]:
    return dict(_EXTRACTORS)


def register(
    extractor: DynamicSymbol,
) -> None:
    """
    Registers a new source code extractor for a given language.

    Parameters:
        language: The name of the language (e.g., "C", "Python") to associate with the extractor.
        class_path: The full import path to the extractor class.
        order: Optional order for insertion. If 'first', prepends the extractor; if 'last', appends it.
    """
    extractor_type: Type[Extractor] = dynamic_import(extractor)
    language = extractor_type.language()
    logger.info(f"Registering a {language} extractor at {extractor}")
    if _EXTRACTORS.setdefault(language.lower(), extractor) != extractor:
        raise ValueError(
            f"An extractor is already registered for the {language} language"
        )


def unregister(symbol_or_language: Union[DynamicSymbol, str]) -> None:
    if isinstance(symbol_or_language, str):
        key = symbol_or_language
    else:
        (key,) = [k for k, v in _EXTRACTORS.items() if v == symbol_or_language]
    extractor = _EXTRACTORS[key]
    del _EXTRACTORS[key]
    logger.info(f"Unregistered {key} extractor at {extractor}")


def unregister_all() -> None:
    _EXTRACTORS.clear()
    logger.info("Unregistered all extractors")


class Extractor(ABC):
    """
    Abstract base class for source code extractors.

    Extractors are responsible for parsing source code files and returning extracted function
    definitions as `SourceFunction` instances.
    """

    @abstractmethod
    def extract(
        self, file_path: PathLike, repo_path: Optional[PathLike] = None
    ) -> Sequence[SourceFunction]:
        """
        Extracts functions from the given source code file.

        Parameters:
            file_path: The path to the source file.
            repo_path: Optional repository root path to calculate relative function scopes.

        Returns:
            A sequence of `SourceFunction` instances extracted from the file.
        """
        pass

    @abstractmethod
    def get_extractable_files(self, path: PathLike) -> Set[Path]:
        """
        Retrieves all files that can be processed by the extractor from the given path.

        Parameters:
            path: A file or directory path to search for extractable files.

        Returns:
            A sequence of `Path` objects representing extractable source files.
        """
        pass

    @staticmethod
    def is_installed() -> bool:
        return True

    @classmethod
    def language(cls) -> str:
        return cls.__name__.replace("Extractor", "")


def create_extractor(language: str, *args: Any, **kwargs: Any) -> Extractor:
    """
    Retrieves the registered extractor instance for the specified language.

    Parameters:
        language: The name of the language for which to retrieve an extractor.
        *args: Positional arguments passed to the extractor's constructor.
        **kwargs: Keyword arguments passed to the extractor's constructor.

    Returns:
        An instance of the extractor class for the given language.

    Raises:
        ExtractorNotFound: If no extractor is registered for the specified language.
    """
    if language in _EXTRACTORS:
        extractor_class: Type[Extractor] = dynamic_import(_EXTRACTORS[language])
        return extractor_class(*args, **kwargs)
    raise ValueError(f'"{language}" is not a registered extractor')


Transform = Callable[[SourceFunction], SourceFunction]
"""
A callable object that transforms a source code function into another source code function.
"""


@dataclass(frozen=True)
class ExtractConfig:
    """
    Configuration for extracting source code functions.
    """

    max_workers: Optional[int] = None
    """
    Maximum number of files to extract functions in parallel.
    """
    accurate_progress: bool = True
    """
    Whether to accurately track progress by counting extractable files in advance. This may take
    longer to start but provides more accurate progress tracking.
    """
    transform: Optional[DynamicSymbol] = None
    """
    An optional transformation to apply to each source code function.
    """
    exclusive_subpaths: Set[Path] = field(default_factory=set)
    """
    A set of subpaths to exclusively extract functions from. If specified, only these subpaths will be extracted.
    """
    exclude_subpaths: Set[Path] = field(default_factory=set)
    """
    A set of subpaths to exclude from extraction. If specified, these subpaths will be ignored.
    """
    checkpoint: int = 10
    """
    The number of steps between saving checkpoints. Set to 0 to disable checkpoints.
    """
    use_checkpoint: bool = True
    """
    `True` if a checkpoint file should be loaded and used to resume extraction.
    """
    extract_as_repo: bool = True
    """
    `True` if the path should be treated as a repository root for calculating relative function scopes.
    """
    extractor_args: Dict[str, Sequence[Any]] = field(default_factory=dict)
    """
    Positional arguments to pass to the extractor's `__init__` method. The keys are language
    names. The values are sequences of arguments. For example, `{'C': [arg1, arg2]}`.
    """
    extractor_kwargs: Dict[str, Dict[str, Any]] = field(default_factory=dict)
    """
    Keyword arguments to pass to the extractor's `__init__` method. The keys are language names.
    The values are dictionaries of keyword arguments. For example, `{'C': {'kwarg1': value1}}`.
    """
    strict: bool = False

    def __post_init__(self) -> None:
        if self.max_workers and self.max_workers < 1:
            raise ValueError("Max workers must be a positive integer")
        if self.exclude_subpaths & self.exclusive_subpaths:
            raise ValueError(
                "Cannot have overlapping paths in exclude_subpaths and "
                "exclusive_subpaths"
            )
        if self.checkpoint < 0:
            raise ValueError("Checkpoint must be a non-negative integer")

    def get_transform(self) -> Optional[Transform]:
        if self.transform:
            return dynamic_import(self.transform)


@codablellm_low_level_task(name="extract_file")
def extract_file_task(
    extractor: Extractor, file: PathLike, repo_path: Optional[PathLike]
) -> Sequence[SourceFunction]:
    return extractor.extract(file, repo_path=repo_path)


@codablellm_low_level_task(name="apply_transform")
def apply_transform_task(
    transform: DynamicSymbol, source: SourceFunction
) -> SourceFunction:
    transform_func: Transform = dynamic_import(transform)
    return transform_func(source)


@codablellm_task(name="extract_directory")
def extract_directory_task(
    path: PathLike, config: ExtractConfig = ExtractConfig()
) -> List[SourceFunction]:
    """
    Extracts source functions from the given path using the specified configuration.

    If `as_callable_pool` is `True`, returns a deferred callable extractor that can be executed later,
    typically used for progress bar display or asynchronous processing.

    Parameters:
        path: The file or directory path from which to extract functions.
        config: Extraction configuration options.
        as_callable_pool: If `True`, returns a callable extractor for deferred execution.

    Returns:
        Either a list of extracted `SourceFunction` instances or a `_CallableExtractor` for deferred execution.
    """
    # Collect extractable files
    logger.info("Collecting extractable source code files...")
    file_extractor_map: Dict[Path, Extractor] = {}
    for language, _ in get_registered():
        extractor = create_extractor(
            language,
            *config.extractor_args.get(language, []),
            **config.extractor_kwargs.get(language, {}),
        )
        # Locate extractable files
        files = extractor.get_extractable_files(path)
        if not any(files):
            logger.debug(f"No {language} files were located")
        elif not extractor.is_installed():
            logger.warning(
                f"{len(files)} {language} files were located, but the built-in {language} "
                "extractor is not installed. You can install support for all languages with "
                "'pip install codablellm[langs]', or you can install this extra individually."
            )
        else:
            for file in files:
                if file_extractor_map.setdefault(file, extractor) != extractor:
                    logger.info(f"Extractor was already specified for {file.name}")
    if not any(file_extractor_map):
        logger.warning("No source code files found to extract")
    # Submit extraction tasks
    logger.info("Submitting extraction tasks...")
    futures = [
        extract_file_task.submit(extractor, file, repo_path=path)
        for file, extractor in file_extractor_map.items()
    ]
    results = [future.result(raise_on_failure=config.strict) for future in futures]
    functions: List[SourceFunction] = []
    for result in results:
        if isinstance(result, list):
            functions.extend(result)
    transform = config.get_transform()
    if transform:
        # Apply transformation
        logger.info("Applying transformation...")
        functions = apply_transform_task.map(functions).result()
    logger.info(f"Successfully extracted {len(functions)} functions")
    return functions


@codablellm_flow()
def extract(
    *paths: PathLike, config: ExtractConfig = ExtractConfig()
) -> List[SourceFunction]:
    futures = [extract_directory_task.submit(path, config=config) for path in paths]
    return [function for future in futures for function in future.result()]


# Register builtin extractors that are installed
for extractor in [
    DynamicSymbol.from_deferred_import("codablellm.languages.c.CExtractor"),
    DynamicSymbol.from_deferred_import("codablellm.languages.cpp.CPPExtractor"),
    DynamicSymbol.from_deferred_import("codablellm.languages.java.JavaExtractor"),
    DynamicSymbol.from_deferred_import(
        "codablellm.languages.javascript.JavaScriptExtractor"
    ),
    DynamicSymbol.from_deferred_import(
        "codablellm.languages.python_language.PythonExtractor"
    ),
    DynamicSymbol.from_deferred_import("codablellm.languages.rust.RustExtractor"),
    DynamicSymbol.from_deferred_import(
        "codablellm.languages.typescript.TypeScriptExtractor"
    ),
]:
    extractor_type: Type[Extractor] = dynamic_import(extractor)
    if extractor_type.is_installed():
        register(extractor)
