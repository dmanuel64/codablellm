"""
The codablellm command line interface.
"""

import logging
import os
import shlex
from enum import Enum
from pathlib import Path
from typing import Final, List, Optional, Type

from click import BadParameter
from rich import print
from typer import Argument, Exit, Option, Typer

import codablellm
import codablellm.logging_config
from codablellm import container
from codablellm.core import downloader
from codablellm.core.decompiler import DecompileConfig
from codablellm.core.extractor import ExtractConfig, Extractor
from codablellm.core.utils import (
    CODABLELLM_MAX_WORKERS_ENVIRON_KEY,
    CODABLELLM_PARALLEL_TASKS_ENVIRON_KEY,
    DynamicSymbol,
)
from codablellm.dataset import DecompiledCodeDatasetConfig, SourceCodeDatasetConfig
from codablellm.decompilers import *
from codablellm.languages import *
from codablellm.repoman import ManageConfig

logger = logging.getLogger(__name__)

app = Typer()

# Argument/option choices


class ExtractorConfigOperation(str, Enum):
    PREPEND = "prepend"
    APPEND = "append"
    SET = "set"


class GenerationMode(str, Enum):
    PATH = "path"
    TEMP = "temp"
    TEMP_APPEND = "temp-append"


class CommandErrorHandler(str, Enum):
    INTERACTIVE = "interactive"
    IGNORE = "ignore"
    NONE = "none"


class RunFrom(str, Enum):
    CWD = "cwd"
    REPO = "repo"


class SymbolRemover(str, Enum):
    STRIP = "strip"
    PSEUDO_STRIP = "pseudo-strip"


# Default configurations


DEFAULT_SOURCE_CODE_DATASET_CONFIG: Final[SourceCodeDatasetConfig] = (
    SourceCodeDatasetConfig(log_generation_warning=False)
)
DEFAULT_DECOMPILED_CODE_DATASET_CONFIG: Final[DecompiledCodeDatasetConfig] = (
    DecompiledCodeDatasetConfig()
)
DEFAULT_MANAGE_CONFIG: Final[ManageConfig] = ManageConfig()

# Argument/option validation callbacks


def validate_dataset_format(path: Path) -> Path:
    if path.suffix.casefold() not in [
        e.casefold()
        for e in [
            ".json",
            ".jsonl",
            ".csv",
            ".tsv",
            ".xlsx",
            ".xls",
            ".xlsm",
            ".md",
            ".markdown",
            ".tex",
            ".html",
            ".html",
            ".xml",
        ]
    ]:
        raise BadParameter(f'Unsupported dataset format: "{path.suffix}"')
    return path


# Miscellaneous argument/option callbacks


def toggle_verbose_logging(enable: bool) -> None:
    logging.getLogger("prefect").setLevel(logging.INFO if enable else logging.WARNING)


def toggle_debug_logging(enable: bool) -> None:
    if enable:
        toggle_verbose_logging(True)
        codablellm.logging_config.setup_logger(logging.DEBUG)


def show_version(show: bool) -> None:
    if show:
        print(f"[b]codablellm {codablellm.__version__}")
        raise Exit()


def try_create_repo_dir(path: Path) -> Path:
    Path(path).mkdir(parents=True, exist_ok=True)
    return path


# Argument/option parsers


def parse_builtin_or_dynamic_symbol(value: str) -> DynamicSymbol:
    # Check for common classes/functions for decompilers/extractors/mappers
    def verify_language_support(extractor_type: Type[Extractor], extra: str) -> None:
        if not extractor_type.is_installed():
            raise BadParameter(
                f'{extractor_type.language()} language support requires the "{extra}" extra to be installed. '
                f'Install with "pip install codablellm[{extra}]" or "pip install codablellm[langs]" to '
                "install support for all languages."
            )

    value = value.lower()
    if value == "ghidra":
        symbol = DynamicSymbol.from_builtin_symbol(Ghidra)
    elif value == "c":
        symbol = DynamicSymbol.from_builtin_symbol(CExtractor)
    elif value == "c++":
        symbol = DynamicSymbol.from_builtin_symbol(CPPExtractor)
    elif value == "java":
        symbol = DynamicSymbol.from_builtin_symbol(JavaExtractor)
        verify_language_support(JavaExtractor, "java")
    elif value == "javascript":
        symbol = DynamicSymbol.from_builtin_symbol(JavaScriptExtractor)
        verify_language_support(JavaExtractor, "javascript")
    elif value == "python":
        symbol = DynamicSymbol.from_builtin_symbol(PythonExtractor)
        verify_language_support(JavaExtractor, "python")
    elif value == "rust":
        symbol = DynamicSymbol.from_builtin_symbol(RustExtractor)
        verify_language_support(JavaExtractor, "rust")
    elif value == "typescript":
        symbol = DynamicSymbol.from_builtin_symbol(TypeScriptExtractor)
        verify_language_support(JavaExtractor, "typescript")
    else:
        try:
            symbol = DynamicSymbol.from_str(value)
        except ValueError as e:
            raise BadParameter(
                "Class/function must be in the format of 'path/to/file.py::ClassOrFunction'"
            ) from e
    return symbol


# Arguments
REPO: Final[Path] = Argument(
    file_okay=False,
    show_default=False,
    callback=try_create_repo_dir,
    help="Path to the local repository.",
)
SAVE_AS: Final[Path] = Argument(
    dir_okay=False,
    show_default=False,
    callback=validate_dataset_format,
    help="Path to save the dataset at.",
)
BINS: Final[Optional[List[Path]]] = Argument(
    None,
    metavar="[PATH]...",
    show_default=False,
    help="List of files or a directories containing the "
    "repository's compiled binaries.",
)

# Options
ACCURATE: Final[bool] = Option(
    DEFAULT_SOURCE_CODE_DATASET_CONFIG.extract_config.accurate_progress,
    "--accurate / --lazy",
    help="Displays estimated time remaining and detailed "
    "progress reporting of source function extraction "
    "if --accurate is enabled, at a cost of more "
    "memory usage and a longer startup time to collect "
    "the sequence of source code files.",
)
BUILD: Final[Optional[str]] = Option(
    None,
    "--build",
    "-b",
    metavar="COMMAND",
    help="If --decompile is specified, the repository will be "
    "built using the value of this option as the build command.",
)
CHECKPOINT: Final[int] = Option(
    DEFAULT_SOURCE_CODE_DATASET_CONFIG.extract_config.checkpoint,
    min=0,
    help="Number of extraction entries after which a backup dataset "
    "file will be saved in case of a crash.",
)
CLEANUP: Final[Optional[str]] = Option(
    DEFAULT_MANAGE_CONFIG.cleanup_command,
    "--cleanup",
    "-c",
    metavar="COMMAND",
    help="If --decompile is specified, the repository will be "
    "cleaned up after the dataset is created, using the value of "
    "this option as the build command.",
)
CLEAR_EXTRACTORS: Final[bool] = Option(
    False,
    "--clear-extractors",
    help="Unregister all builtin extractors.",
    callback=codablellm.extractor.unregister_all,
    parser=parse_builtin_or_dynamic_symbol,
    metavar="SYMBOL",
)
CONTAINERIZE: Final[bool] = Option(
    False,
    "--containerize / --local",
    "-C / -l",
    help="Run inside a Docker container instead of the local environment.",
)
DECOMPILE: Final[bool] = Option(
    False,
    "--decompile / --source",
    "-d / -s",
    help="If the language supports decompiled code mapping, use "
    "--decompiler to decompile the binaries specified by the bins "
    "argument and add decompiled code to the dataset.",
)
DECOMPILER: Final[DynamicSymbol] = Option(
    str(codablellm.decompiler.get()),
    help="Decompiler to use.",
    parser=parse_builtin_or_dynamic_symbol,
    metavar="SYMBOL",
)
DEBUG: Final[bool] = Option(
    False, "--debug", callback=toggle_debug_logging, hidden=True
)
EXCLUDE_SUBPATH: Final[Optional[List[Path]]] = Option(
    list(DEFAULT_SOURCE_CODE_DATASET_CONFIG.extract_config.exclude_subpaths),
    "--exclude-subpath",
    "-e",
    help="Path relative to the repository "
    "directory to exclude from the dataset "
    "generation.",
)
EXCLUSIVE_SUBPATH: Final[Optional[List[Path]]] = Option(
    list(DEFAULT_SOURCE_CODE_DATASET_CONFIG.extract_config.exclusive_subpaths),
    "--exclusive-subpath",
    "-E",
    help="Path relative to the repository "
    "directory to exclusively include in the dataset "
    "generation.",
)
EXTRA_PATH: Final[List[Path]] = Option(
    [],
    exists=True,
    help="Extra files/directories to add to the repository (e.g. build scripts).",
)
GENERATION_MODE: Final[GenerationMode] = Option(
    DEFAULT_SOURCE_CODE_DATASET_CONFIG.generation_mode,
    help="Specify how the dataset should be generated from the repository.",
)
GHIDRA: Final[Optional[Path]] = Option(
    Ghidra.get_path(),
    envvar=Ghidra.ENVIRON_KEY,
    dir_okay=False,
    callback=lambda v: Ghidra.set_path(v) if v else None,
    help="Path to Ghidra's analyzeHeadless command.",
)
GHIDRA_SCRIPT: Final[Path] = Option(
    Ghidra.get_decompile_script(),
    dir_okay=False,
    exists=True,
    callback=lambda v: Ghidra.set_decompile_script(v),
    help="Path to the decompile script for Ghidra that serialzies a DecompiledFunctionJSONObject",
)
GIT: Final[bool] = Option(
    False,
    "--git / --archive",
    help="Determines whether --url is a Git "
    "download URL or a tarball/zipfile download URL.",
)
BUILD_ERROR_HANDLING: Final[CommandErrorHandler] = Option(
    DEFAULT_MANAGE_CONFIG.build_error_handling,
    help="Specifies how to handle errors that occur "
    "during the cleanup process. Options include "
    "ignoring the error, raising an exception, or "
    "prompting the user for manual intervention.",
)
CLEANUP_ERROR_HANDLING: Final[CommandErrorHandler] = Option(
    DEFAULT_MANAGE_CONFIG.cleanup_error_handling,
    help="Specifies how to handle errors that occur "
    "during the cleanup process. Options include "
    "ignoring the error, raising an exception, or "
    "prompting the user for manual intervention.",
)
MAPPER: Final[DynamicSymbol] = Option(
    str(DEFAULT_DECOMPILED_CODE_DATASET_CONFIG.mapper),
    parser=parse_builtin_or_dynamic_symbol,
    metavar="SYMBOL",
    help="Mapper to use for mapping decompiled functions to source code functions.",
)
MAX_WORKERS: Final[Optional[int]] = Option(
    None,
    callback=lambda v: (
        os.environ.update({CODABLELLM_MAX_WORKERS_ENVIRON_KEY: str(v)}) if v else None
    ),
    min=1,
    envvar=CODABLELLM_MAX_WORKERS_ENVIRON_KEY,
    help="Maximum number of processes/threads for prefect tasks.",
)
PARALLEL: Final[bool] = Option(
    False,
    "--parallel / --concurrent",
    callback=lambda v: os.environ.update(
        {CODABLELLM_PARALLEL_TASKS_ENVIRON_KEY: str(v)}
    ),
    envvar=CODABLELLM_PARALLEL_TASKS_ENVIRON_KEY,
    help="If CodableLLM should execute prefect tasks in parallel or concurrently",
)
VERBOSE: Final[bool] = Option(
    False,
    "--verbose",
    "-v",
    callback=toggle_verbose_logging,
    help="Display verbose logging information.",
)
VERSION: Final[bool] = Option(
    False,
    "--version",
    is_eager=True,
    callback=show_version,
    help="Shows the installed version of codablellm and exit.",
)
STRICT: Final[bool] = Option(
    DEFAULT_SOURCE_CODE_DATASET_CONFIG.extract_config.strict
    or DEFAULT_DECOMPILED_CODE_DATASET_CONFIG.decompiler_config.strict,
    "--strict",
    help="Crash if an extraction or decompilation fails.",
)
SYMBOL_REMOVER: Final[Optional[SymbolRemover]] = Option(
    DEFAULT_DECOMPILED_CODE_DATASET_CONFIG.decompiler_config.symbol_remover,
    help="If a decompiled dataset is being created, strip the symbols "
    "after decompiling",
)
TRANSFORM: Final[Optional[DynamicSymbol]] = Option(
    DEFAULT_SOURCE_CODE_DATASET_CONFIG.extract_config.transform,
    "--transform",
    "-t",
    parser=parse_builtin_or_dynamic_symbol,
    metavar="SYMBOL",
    help="Transformation function to use when extracting source code functions.",
)
RECURSIVE: Final[bool] = Option(
    DEFAULT_DECOMPILED_CODE_DATASET_CONFIG.decompiler_config.recursive,
    "--recursive",
    "-r",
    help="Recursively search for binaries in the specified bins directories.",
)
REGISTER_EXTRACTOR: Final[Optional[List[DynamicSymbol]]] = Option(
    [],
    "--register-extractor",
    "-R",
    parser=parse_builtin_or_dynamic_symbol,
    metavar="SYMBOL",
    help="Additional extractor to register.",
)
RUN_FROM: Final[RunFrom] = Option(
    DEFAULT_MANAGE_CONFIG.run_from,
    help="Where to run build/clean commands from: 'repo' (the root "
    "of the repository, whether real or temp) or 'cwd' (your "
    "current shell directory). Useful for managing relative path behavior.",
)
USE_CHECKPOINT: Final[Optional[bool]] = Option(
    None,
    "--use-checkpoint / --ignore-checkpoint",
    show_default=False,
    help="Enable the use of an extraction checkpoint "
    "to resume from a previously saved state.",
)
URL: Final[str] = Option(
    "",
    help="Download a remote repository and save at the local path "
    "specified by the REPO argument.",
)


@app.command()
def command(
    repo: Path = REPO,
    save_as: Path = SAVE_AS,
    bins: Optional[List[Path]] = BINS,
    accurate: bool = ACCURATE,
    build: Optional[str] = BUILD,
    build_error_handling: CommandErrorHandler = BUILD_ERROR_HANDLING,
    cleanup: Optional[str] = CLEANUP,
    cleanup_error_handling: CommandErrorHandler = CLEANUP_ERROR_HANDLING,
    containerize: bool = CONTAINERIZE,
    checkpoint: int = CHECKPOINT,
    _debug: bool = DEBUG,
    decompile: bool = DECOMPILE,
    decompiler: DynamicSymbol = DECOMPILER,
    exclude_subpath: Optional[List[Path]] = EXCLUDE_SUBPATH,
    exclusive_subpath: Optional[List[Path]] = EXCLUSIVE_SUBPATH,
    extra_path: List[Path] = EXTRA_PATH,
    generation_mode: GenerationMode = GENERATION_MODE,
    git: bool = GIT,
    _ghidra: Optional[Path] = GHIDRA,
    _ghidra_script: Path = GHIDRA_SCRIPT,
    mapper: DynamicSymbol = MAPPER,
    max_workers: Optional[int] = MAX_WORKERS,
    parallel: bool = PARALLEL,
    recursive: bool = RECURSIVE,
    run_from: RunFrom = RUN_FROM,
    strict: bool = STRICT,
    symbol_remover: Optional[SymbolRemover] = SYMBOL_REMOVER,
    transform: Optional[DynamicSymbol] = TRANSFORM,
    use_checkpoint: Optional[bool] = USE_CHECKPOINT,
    url: str = URL,
    _verbose: bool = VERBOSE,
    _version: bool = VERSION,
) -> None:
    """
    Creates a code dataset from a local repository.
    """
    if containerize:
        container.run_containerized(save_as)
        return
    if decompiler != codablellm.decompiler.get().symbol:
        # Configure decompiler
        codablellm.decompiler.set(decompiler)
    if url:
        # Download remote repository
        if git:
            downloader.clone(url, repo)
        else:
            downloader.decompress(url, repo)
    # Create the extractor configuration
    extract_config = ExtractConfig(
        accurate_progress=accurate,
        transform=transform,
        exclusive_subpaths=set(exclusive_subpath) if exclusive_subpath else set(),
        exclude_subpaths=set(exclude_subpath) if exclude_subpath else set(),
        checkpoint=checkpoint,
        use_checkpoint=True,
        strict=strict,
    )
    if build:
        logger.warning(
            "--build specified without --decompile. --decompile enabled "
            "automatically."
        )
        decompile = True
    # Create source code/decompiled code dataset
    if decompile:
        if not bins or not any(bins):
            raise BadParameter(
                "Must specify at least one binary for decompiled code datasets.",
                param_hint="bins",
            )
        dataset_config = DecompiledCodeDatasetConfig(
            extract_config=extract_config,
            decompiler_config=DecompileConfig(
                symbol_remover=symbol_remover,  # type: ignore
                recursive=recursive,
                strict=strict,
            ),
            mapper=mapper,
        )
        if not build:
            dataset = codablellm.create_decompiled_dataset(
                repo, bins, extract_config=extract_config, dataset_config=dataset_config
            )
        else:
            manage_config = ManageConfig(
                cleanup_command=shlex.split(cleanup) if cleanup else None,
                run_from=run_from,  # type: ignore
                build_error_handling=build_error_handling,  # type: ignore
                cleanup_error_handling=cleanup_error_handling,  # type: ignore
                extra_paths=extra_path,
            )
            dataset = codablellm.compile_dataset(
                repo,
                bins,
                shlex.split(build),
                manage_config=manage_config,
                extract_config=extract_config,
                dataset_config=dataset_config,
                generation_mode=generation_mode,  # type: ignore
            )
    else:
        dataset_config = SourceCodeDatasetConfig(
            generation_mode=str(generation_mode),  # type: ignore
            extract_config=extract_config,
        )
        dataset = codablellm.create_source_dataset(repo, config=dataset_config)
    # Save dataset
    dataset.save_as(save_as)
