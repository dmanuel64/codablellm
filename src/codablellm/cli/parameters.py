from enum import Enum
from pathlib import Path
from typing import Final, List, Optional
from codablellm import ManageConfig
from typer import Argument, Option
from typing_extensions import Annotated

from codablellm.cli import processor
from codablellm.core.utils import (
    CODABLELLM_MAX_WORKERS_ENVIRON_KEY,
    CODABLELLM_PARALLEL_TASKS_ENVIRON_KEY,
    DynamicSymbol,
)
from codablellm.dataset import DecompiledCodeDatasetConfig, SourceCodeDatasetConfig
from codablellm import decompiler
from codablellm.decompilers.ghidra import Ghidra


# Choices
class GenerationModeChoice(str, Enum):
    PATH = "path"
    TEMP = "temp"
    TEMP_APPEND = "temp-append"


class CommandErrorHandlerChoice(str, Enum):
    INTERACTIVE = "interactive"
    IGNORE = "ignore"
    NONE = "none"


class RunFromChoice(str, Enum):
    CWD = "cwd"
    REPO = "repo"


class SymbolRemoverChoice(str, Enum):
    STRIP = "strip"
    PSEUDO_STRIP = "pseudo-strip"


# Arguments
RepoArg = Annotated[
    Path,
    Argument(
        file_okay=False,
        show_default=False,
        # callback=try_create_repo_dir,
        help="Path to the local repository.",
    ),
]
SaveAsArg = Annotated[
    Path,
    Argument(
        dir_okay=False,
        show_default=False,
        callback=processor.validate_dataset_format,
        help="Path to save the dataset at.",
    ),
]
BinsArg = Annotated[
    Optional[List[Path]],
    Argument(
        metavar="[PATH]...",
        show_default=False,
        help="List of files or a directories containing the "
        "repository's compiled binaries.",
    ),
]

# Options
BuildOpt = Annotated[
    Optional[str],
    Option(
        ...,
        "--build",
        "-b",
        metavar="COMMAND",
        rich_help_panel="Utils and Configs",
        help="If --decompile is specified, the repository will be "
        "built using the value of this option as the build command.",
    ),
]
BuildErrorHandlerOpt = Annotated[
    CommandErrorHandlerChoice,
    Option(
        ...,
        help="Specifies how to handle errors that occur "
        "during the cleanup process. Options include "
        "ignoring the error, raising an exception, or "
        "prompting the user for manual intervention.",
    ),
]
CleanupOpt = Annotated[
    Optional[str],
    Option(
        ...,
        "--cleanup",
        "-c",
        metavar="COMMAND",
        help="If --decompile is specified, the repository will be "
        "cleaned up after the dataset is created, using the value of "
        "this option as the build command.",
    ),
]
CleanupErrorHandlerOpt = Annotated[
    CommandErrorHandlerChoice,
    Option(
        ...,
        help="Specifies how to handle errors that occur "
        "during the cleanup process. Options include "
        "ignoring the error, raising an exception, or "
        "prompting the user for manual intervention.",
    ),
]
ClearExtractorsOpt = Annotated[
    bool,
    Option(
        ...,
        "--clear-extractors",
        help="Unregister all builtin extractors.",
        # callback=codablellm.extractor.unregister_all,
    ),
]
ContainerizeOpt = Annotated[
    bool,
    Option(
        ...,
        "--containerize / --local",
        "-C / -l",
        help="Run inside a Docker container instead of the local environment.",
    ),
]
DecompileOpt = Annotated[
    bool,
    Option(
        ...,
        "--decompile / --source",
        "-d / -s",
        help="If the language supports decompiled code mapping, use "
        "--decompiler to decompile the binaries specified by the bins "
        "argument and add decompiled code to the dataset.",
    ),
]
DecompilerOpt = Annotated[
    DynamicSymbol,
    Option(
        ...,
        help="Decompiler to use.",
        parser=processor.parse_symbol,
        metavar="SYMBOL",
    ),
]
DebugOpt = Annotated[
    bool,
    Option(
        ...,
        "--debug",
        # callback=toggle_debug_logging,
        hidden=True,
    ),
]
ExtraPathOpt = Annotated[
    Optional[List[Path]],
    Option(
        ...,
        exists=True,
        help="Extra files/directories to add to the repository (e.g. build scripts).",
    ),
]
GenerationModeOpt = Annotated[
    GenerationModeChoice,
    Option(
        ...,
        help="Specify how the dataset should be generated from the repository.",
    ),
]
GhidraOpt = Annotated[
    Optional[Path],
    Option(
        ...,
        envvar=Ghidra.ENVIRON_KEY,
        dir_okay=False,
        # callback=lambda v: Ghidra.set_path(v) if v else None,
        help="Path to Ghidra's analyzeHeadless command.",
    ),
]
GhidraScriptOpt = Annotated[
    Path,
    Option(
        ...,
        dir_okay=False,
        exists=True,
        # callback=lambda v: Ghidra.set_decompile_script(v),
        help="Path to the decompile script for Ghidra that serialzies a DecompiledFunctionJSONObject",
    ),
]
GitOpt = Annotated[
    bool,
    Option(
        ...,
        "--git / --archive",
        help="Determines whether --url is a Git "
        "download URL or a tarball/zipfile download URL.",
    ),
]
MapperOpt = Annotated[
    DynamicSymbol,
    Option(
        ...,
        parser=processor.parse_symbol,
        metavar="SYMBOL",
        help="Mapper to use for mapping decompiled functions to source code functions.",
    ),
]
MaxWorkersOpt = Annotated[
    Optional[int],
    Option(
        ...,
        # callback=lambda v: (
        #     os.environ.update({CODABLELLM_MAX_WORKERS_ENVIRON_KEY: str(v)}) if v else None
        # ),
        min=1,
        envvar=CODABLELLM_MAX_WORKERS_ENVIRON_KEY,
        help="Maximum number of processes/threads for prefect tasks.",
    ),
]
ParallelOpt = Annotated[
    bool,
    Option(
        ...,
        "--parallel / --concurrent",
        # callback=lambda v: os.environ.update(
        #     {CODABLELLM_PARALLEL_TASKS_ENVIRON_KEY: str(v)}
        # ),
        envvar=CODABLELLM_PARALLEL_TASKS_ENVIRON_KEY,
        help="If CodableLLM should execute prefect tasks in parallel or concurrently",
    ),
]
VerboseOpt = Annotated[
    bool,
    Option(
        ...,
        "--verbose",
        "-v",
        # callback=toggle_verbose_logging,
        help="Display verbose logging information.",
    ),
]
VersionOpt = Annotated[
    bool,
    Option(
        ...,
        "--version",
        is_eager=True,
        callback=processor.show_version,
        help="Shows the installed version of codablellm and exit.",
    ),
]
StrictOpt = Annotated[
    bool,
    Option(
        ...,
        "--strict",
        help="Crash if an extraction or decompilation fails.",
    ),
]
SymbolRemoverOpt = Annotated[
    Optional[SymbolRemoverChoice],
    Option(
        ...,
        help="If a decompiled dataset is being created, strip the symbols "
        "after decompiling",
    ),
]
TransformOpt = Annotated[
    Optional[DynamicSymbol],
    Option(
        ...,
        "--transform",
        "-t",
        parser=processor.parse_symbol,
        metavar="SYMBOL",
        help="Transformation function to use when extracting source code functions.",
    ),
]
RecursiveOpt = Annotated[
    bool,
    Option(
        ...,
        "--recursive",
        "-r",
        help="Recursively search for binaries in the specified bins directories.",
    ),
]
RegisterExtractorOpt = Annotated[
    Optional[List[DynamicSymbol]],
    Option(
        ...,
        "--register-extractor",
        "-R",
        parser=processor.parse_symbol,
        metavar="SYMBOL",
        help="Additional extractor to register.",
    ),
]
RunFromOpt = Annotated[
    RunFromChoice,
    Option(
        ...,
        help="Where to run build/clean commands from: 'repo' (the root "
        "of the repository, whether real or temp) or 'cwd' (your "
        "current shell directory). Useful for managing relative path behavior.",
    ),
]
UrlOpt = Annotated[
    Optional[str],
    Option(
        ...,
        help="Download a remote repository and save at the local path "
        "specified by the REPO argument.",
    ),
]
