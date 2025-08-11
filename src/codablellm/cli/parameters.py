from pathlib import Path
from typing import List, Optional
from typer import Argument, Option
from typing_extensions import Annotated

from codablellm.core.utils import DynamicSymbol
from codablellm.decompilers.ghidra import Ghidra

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
        callback=validate_dataset_format,
        help="Path to save the dataset at.",
    ),
]
BinsArg = Annotated[
    Optional[List[Path]],
    Argument(
        None,
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
        None,
        "--build",
        "-b",
        metavar="COMMAND",
        help="If --decompile is specified, the repository will be "
        "built using the value of this option as the build command.",
    ),
]
BuildErrorHandlerOpt = Annotated[
    CommandErrorHandlerChoice,
    Option(
        DEFAULT_MANAGE_CONFIG.build_error_handling,
        help="Specifies how to handle errors that occur "
        "during the cleanup process. Options include "
        "ignoring the error, raising an exception, or "
        "prompting the user for manual intervention.",
    ),
]
CleanupOpt = Annotated[
    Optional[str],
    Option(
        DEFAULT_MANAGE_CONFIG.cleanup_command,
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
        DEFAULT_MANAGE_CONFIG.cleanup_error_handling,
        help="Specifies how to handle errors that occur "
        "during the cleanup process. Options include "
        "ignoring the error, raising an exception, or "
        "prompting the user for manual intervention.",
    ),
]
ClearExtractorsOpt = Annotated[
    bool,
    Option(
        False,
        "--clear-extractors",
        help="Unregister all builtin extractors.",
        # callback=codablellm.extractor.unregister_all,
    ),
]
ContainerizeOpt = Annotated[
    bool,
    Option(
        False,
        "--containerize / --local",
        "-C / -l",
        help="Run inside a Docker container instead of the local environment.",
    ),
]
DecompileOpt = Annotated[
    bool,
    Option(
        False,
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
        str(codablellm.decompiler.get()),
        help="Decompiler to use.",
        parser=parse_builtin_or_dynamic_symbol,
        metavar="SYMBOL",
    ),
]
DebugOpt = Annotated[
    bool,
    Option(
        False,
        "--debug",
        # callback=toggle_debug_logging,
        hidden=True,
    ),
]
ExtraPathOpt = Annotated[
    Optional[List[Path]],
    Option(
        None,
        exists=True,
        help="Extra files/directories to add to the repository (e.g. build scripts).",
    ),
]
GenerationModeOpt = Annotated[
    GenerationModeChoice,
    Option(
        DEFAULT_SOURCE_CODE_DATASET_CONFIG.generation_mode,
        help="Specify how the dataset should be generated from the repository.",
    ),
]
GhidraOpt = Annotated[
    Optional[Path],
    Option(
        Ghidra.get_path(),
        envvar=Ghidra.ENVIRON_KEY,
        dir_okay=False,
        # callback=lambda v: Ghidra.set_path(v) if v else None,
        help="Path to Ghidra's analyzeHeadless command.",
    ),
]
GhidraScriptOpt = Annotated[
    Path,
    Option(
        Ghidra.get_decompile_script(),
        dir_okay=False,
        exists=True,
        # callback=lambda v: Ghidra.set_decompile_script(v),
        help="Path to the decompile script for Ghidra that serialzies a DecompiledFunctionJSONObject",
    ),
]
GitOpt = Annotated[
    bool,
    Option(
        False,
        "--git / --archive",
        help="Determines whether --url is a Git "
        "download URL or a tarball/zipfile download URL.",
    ),
]
MapperOpt = Annotated[
    DynamicSymbol,
    Option(
        str(DEFAULT_DECOMPILED_CODE_DATASET_CONFIG.mapper),
        parser=parse_builtin_or_dynamic_symbol,
        metavar="SYMBOL",
        help="Mapper to use for mapping decompiled functions to source code functions.",
    ),
]
MaxWorkersOpt = Annotated[
    None,
    Option(
        None,
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
        False,
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
        False,
        "--verbose",
        "-v",
        # callback=toggle_verbose_logging,
        help="Display verbose logging information.",
    ),
]
VersionOpt = Annotated[
    bool,
    Option(
        False,
        "--version",
        is_eager=True,
        callback=show_version,
        help="Shows the installed version of codablellm and exit.",
    ),
]
StrictOpt = Annotated[
    bool,
    Option(
        DEFAULT_SOURCE_CODE_DATASET_CONFIG.extract_config.strict
        or DEFAULT_DECOMPILED_CODE_DATASET_CONFIG.decompiler_config.strict,
        "--strict",
        help="Crash if an extraction or decompilation fails.",
    ),
]
SymbolRemoverOpt = Annotated[
    Optional[SymbolRemoverChoice],
    Option(
        DEFAULT_DECOMPILED_CODE_DATASET_CONFIG.decompiler_config.symbol_remover,
        help="If a decompiled dataset is being created, strip the symbols "
        "after decompiling",
    ),
]
TransformOpt = Annotated[
    Optional[DynamicSymbol],
    Option(
        DEFAULT_SOURCE_CODE_DATASET_CONFIG.extract_config.transform,
        "--transform",
        "-t",
        parser=parse_builtin_or_dynamic_symbol,
        metavar="SYMBOL",
        help="Transformation function to use when extracting source code functions.",
    ),
]
RecursiveOpt = Annotated[
    bool,
    Option(
        DEFAULT_DECOMPILED_CODE_DATASET_CONFIG.decompiler_config.recursive,
        "--recursive",
        "-r",
        help="Recursively search for binaries in the specified bins directories.",
    ),
]
RegisterExtractorOpt = Annotated[
    Optional[List[DynamicSymbol]],
    Option(
        None,
        "--register-extractor",
        "-R",
        parser=parse_builtin_or_dynamic_symbol,
        metavar="SYMBOL",
        help="Additional extractor to register.",
    ),
]
RunFromOpt = Annotated[
    RunFromChoice,
    Option(
        DEFAULT_MANAGE_CONFIG.run_from,
        help="Where to run build/clean commands from: 'repo' (the root "
        "of the repository, whether real or temp) or 'cwd' (your "
        "current shell directory). Useful for managing relative path behavior.",
    ),
]
UrlOpt = Annotated[
    Optional[str],
    Option(
        None,
        help="Download a remote repository and save at the local path "
        "specified by the REPO argument.",
    ),
]
