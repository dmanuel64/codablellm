from typing import Final
from typer import Typer

from codablellm.cli import parameters
from codablellm.core import decompiler
from codablellm.dataset import DecompiledCodeDatasetConfig, SourceCodeDatasetConfig
from codablellm.decompilers.ghidra import Ghidra
from codablellm.repoman import ManageConfig

# Default config values
DEFAULT_MANAGE_CONFIG: Final[ManageConfig] = ManageConfig()
DEFAULT_SOURCE_CODE_DATASET_CONFIG: Final[SourceCodeDatasetConfig] = (
    SourceCodeDatasetConfig()
)
DEFAULT_DECOMPILED_CODE_DATASET_CONFIG: Final[DecompiledCodeDatasetConfig] = (
    DecompiledCodeDatasetConfig()
)


app = Typer(rich_markup_mode="markdown")


@app.command()
def command(
    repo: parameters.RepoArg,
    save_as: parameters.SaveAsArg,
    bins: parameters.BinsArg = None,
    build: parameters.BuildOpt = None,
    build_error_handler: parameters.BuildErrorHandlerOpt = DEFAULT_MANAGE_CONFIG.build_error_handling,  # type: ignore
    cleanup: parameters.CleanupOpt = DEFAULT_MANAGE_CONFIG.cleanup_command,  # type: ignore
    cleanup_error_handler: parameters.CleanupErrorHandlerOpt = DEFAULT_MANAGE_CONFIG.cleanup_error_handling,  # type: ignore
    clear_extractors: parameters.ClearExtractorsOpt = False,
    containerize: parameters.ContainerizeOpt = False,
    decompile: parameters.DecompileOpt = False,
    decompiler: parameters.DecompilerOpt = decompiler.get(),
    debug: parameters.DebugOpt = False,
    extra_path: parameters.ExtraPathOpt = None,
    generation_mode: parameters.GenerationModeOpt = DEFAULT_SOURCE_CODE_DATASET_CONFIG.generation_mode,  # type: ignore
    ghidra: parameters.GhidraOpt = Ghidra.get_path(),
    ghidra_script: parameters.GhidraScriptOpt = Ghidra.get_decompile_script(),
    git: parameters.GitOpt = False,
    mapper: parameters.MapperOpt = DEFAULT_DECOMPILED_CODE_DATASET_CONFIG.mapper,
    max_workers: parameters.MaxWorkersOpt = None,
    # parallel: parameters.ParallelOpt = False,
    recursive: parameters.RecursiveOpt = DEFAULT_DECOMPILED_CODE_DATASET_CONFIG.decompiler_config.recursive,
    register_extractor: parameters.RegisterExtractorOpt = None,
    run_from: parameters.RunFromOpt = DEFAULT_MANAGE_CONFIG.run_from,  # type: ignore
    strict: parameters.StrictOpt = DEFAULT_SOURCE_CODE_DATASET_CONFIG.extract_config.strict
    or DEFAULT_DECOMPILED_CODE_DATASET_CONFIG.decompiler_config.strict,
    symbol_remover: parameters.SymbolRemoverOpt = DEFAULT_DECOMPILED_CODE_DATASET_CONFIG.decompiler_config.symbol_remover,  # type: ignore
    transform: parameters.TransformOpt = DEFAULT_SOURCE_CODE_DATASET_CONFIG.extract_config.transform,
    url: parameters.UrlOpt = None,
    verbose: parameters.VerboseOpt = False,
    _version: parameters.VersionOpt = False,
):
    """
    **Create** a new *shinny* user. :sparkles:

    * Create a username

    * Show that the username is created

    ---

    Learn more at the [Typer docs website](https://typer.tiangolo.com)
    """
    print(locals())
