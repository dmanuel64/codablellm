"""
High-level functionality for creating code datasets from source code repositories.
"""

import logging
import shutil
from contextlib import contextmanager, nullcontext
from dataclasses import dataclass, field, replace
from typing import Collection, Generator, Literal, Optional, Sequence, no_type_check


from codablellm.core import utils
from codablellm.core.extractor import ExtractConfig
from codablellm.dataset import (
    DatasetGenerationMode,
    DecompiledCodeDataset,
    DecompiledCodeDatasetConfig,
    SourceCodeDataset,
    SourceCodeDatasetConfig,
)

logger = logging.getLogger(__name__)


def build(
    command: utils.Command,
    error_handler: Optional[utils.CommandErrorHandler] = None,
    show_progress: Optional[bool] = None,
    cwd: Optional[utils.PathLike] = None,
) -> None:
    """
    Builds a local repository using a specified CLI command.

    Parameters:
        command: The CLI command to execute for building the repository.
        error_handler: Specifies how to handle errors during the build process.
        show_progress: Specifies whether to display a progress bar during the build process.
        cwd: The working directory to execute the build command in.
    """
    task = "Building repository..."
    utils.execute_command(
        command,
        task=task,
        **utils.resolve_kwargs(error_handler=error_handler, cwd=cwd),
    )


def cleanup(
    command: utils.Command,
    error_handler: Optional[utils.CommandErrorHandler] = None,
    show_progress: Optional[bool] = None,
    cwd: Optional[utils.PathLike] = None,
) -> None:
    """
    Cleans up build artifacts of a local repository using a specified CLI command.

    Parameters:
        command: The CLI command to execute for cleaning up the repository.
        error_handler: Specifies how to handle errors during the cleanup process.
        show_progress: Specifies whether to display a progress bar during the cleanup process.
        cwd: The working directory to execute the build command in.
    """
    task = "Cleaning up repository..."
    utils.execute_command(
        command,
        task=task,
        **utils.resolve_kwargs(error_handler=error_handler, cwd=cwd),
    )


@dataclass(frozen=True)
class ManageConfig:
    """
    Configuration settings for managing a built local repository.
    """

    cleanup_command: Optional[utils.Command] = None
    """
    An optional CLI command to clean up the build artifacts of the repository.
    """
    build_error_handling: utils.CommandErrorHandler = "interactive"
    """
    Specifies how to handle errors during the build process.
    """
    cleanup_error_handling: utils.CommandErrorHandler = "ignore"
    """
    Specifies how to handle errors during the cleanup process, if `cleanup_command` is provided.
    """
    show_progress: Optional[bool] = None
    """
    Indicates whether to display a progress bar during both the build and cleanup processes. 
    """
    run_from: Literal["cwd", "repo"] = "repo"
    """'
    Specifies the working directory from which to run build and clean commands.

    - `repo`: Use the root of the repository as the working directory. This may refer to the original
    repository path or a duplicated temporary copy depending on the generation mode.
    - `cwd`: Use the current working directory at the time the command is run.

    This option controls how relative paths within commands are resolved and can affect the behavior
    of tools that assume a specific project root.
    """
    extra_paths: Sequence[utils.PathLike] = field(default_factory=list)


@contextmanager
def manage(
    build_command: utils.Command,
    path: utils.PathLike,
    config: ManageConfig = ManageConfig(),
) -> Generator[None, None, None]:
    """
    Builds a local repository and optionally cleans up the build artifacts using a context manager.

    Parameters:
        build_command: The CLI command used to build the repository.
        path: Path to the local repository to manage.
        config: Configuration settings for managing the repository.

    Returns:
        A context manager that builds the repository upon entering and optionally cleans up build artifacts upon exiting, based on the provided configuration.
    """
    # Move extra files to repository
    for extra_path in config.extra_paths:
        shutil.copy(extra_path, path)
    build(
        build_command,
        error_handler=config.build_error_handling,
        cwd=path if config.run_from == "repo" else None,
        show_progress=config.show_progress,
    )
    yield
    if config.cleanup_command:
        cleanup(
            config.cleanup_command,
            error_handler=config.cleanup_error_handling,
            cwd=path if config.run_from == "repo" else None,
            show_progress=config.show_progress,
        )


@no_type_check
@utils.codablellm_flow()
def create_source_dataset(
    path: utils.PathLike,
    config: SourceCodeDatasetConfig = SourceCodeDatasetConfig(
        log_generation_warning=False
    ),
) -> SourceCodeDataset:
    return SourceCodeDataset.from_repository(path, config=config)


@no_type_check
@utils.codablellm_flow()
def create_decompiled_dataset(
    path: utils.PathLike,
    bins: Collection[utils.PathLike],
    extract_config: ExtractConfig = ExtractConfig(),
    dataset_config: DecompiledCodeDatasetConfig = DecompiledCodeDatasetConfig(),
) -> DecompiledCodeDataset:
    return DecompiledCodeDataset.from_repository(
        path, bins, extract_config=extract_config, dataset_config=dataset_config
    )


@utils.codablellm_task(name="compile_dataset", on_completion=[utils.benchmark_task])
def compile_dataset_task(
    path: utils.PathLike,
    bins: Collection[utils.PathLike],
    build_command: utils.Command,
    manage_config: ManageConfig = ManageConfig(),
    extract_config: ExtractConfig = ExtractConfig(),
    dataset_config: DecompiledCodeDatasetConfig = DecompiledCodeDatasetConfig(),
    generation_mode: DatasetGenerationMode = "temp",
) -> DecompiledCodeDataset:
    """
    Builds a local repository and creates a `DecompiledCodeDataset` by decompiling the specified binaries.

    This function automates the process of building a repository, decompiling its binaries,
    and generating a dataset of decompiled functions mapped to their potential source functions.
    It supports flexible configuration for repository management, source code extraction, and
    dataset generation.

    Example:
            ```py
            compile_dataset('path/to/my/repository',
                                [
                                'path/to/my/repository/bin1.exe',
                                'path/to/my/repository/bin2.exe'
                                ],
                                'make',
                                manage_config=ManageConfig(
                                    cleanup_command='make clean'
                                )
                                extract_config=ExtractConfig(
                                    transform=remove_comments
                                ),
                                dataset_config=DecompiledCodeDatasetConfig(
                                    strip=True
                                ),
                                generation_mode='path'
                            )
            ```

            The above example creates a decompiled code dataset from
            `path/to/my/repository`. It removes all comments from the extracted source
            code functions using the specified transform (`remove_comments`), builds the repository
            with `make`, decompiles, the binaries `bin1.exe` and `bin2.exe`, strips symbols after
            decompilation, and finally cleans up the repository with `make clean`.

    Parameters:
        path: Path to the local repository to generate the dataset from.
        bins: A sequence of paths to the built binaries of the repository that should be decompiled.
        build_command: The CLI command used to build the repository.
        manage_config: Configuration settings for managing the repository.
        extract_config: Configuration settings for extracting source code functions.
        dataset_config: Configuration settings for generating the decompiled code dataset.
        generation_mode: Specifies the mode for generating the dataset.

    Returns:
        The generated dataset containing mappings of decompiled functions to their potential source code functions.
    """
    original_path = path
    original_bins = bins
    with utils.prepared_dir(
        path,
        subpaths=bins,
        rebased=generation_mode == "temp" or generation_mode == "temp-append",
    ) as paths:
        path, bins = paths
        # Normalize binaries
        bins = [bins] if isinstance(bins, str) else bins
        # Build repository
        with manage(build_command, path, config=manage_config):
            future = DecompiledCodeDataset.from_repository.submit(
                path, bins, extract_config=extract_config, dataset_config=dataset_config
            )  # type: ignore
            if generation_mode == "temp-append":
                # Create a copy of the extract config to extract the path without a transform
                no_transform_extract_config = replace(extract_config, transform=None)
                original_futures = compile_dataset_task.submit(
                    original_path,
                    original_bins,
                    build_command,
                    manage_config=manage_config,
                    extract_config=no_transform_extract_config,
                    dataset_config=dataset_config,
                    generation_mode="path",
                )
                return DecompiledCodeDataset.create_aligned_dataset(
                    original_futures.result(), future.result()
                )
            return future.result()


@utils.codablellm_flow()
def compile_dataset(
    path: utils.PathLike,
    bins: Collection[utils.PathLike],
    build_command: utils.Command,
    manage_config: ManageConfig = ManageConfig(),
    extract_config: ExtractConfig = ExtractConfig(),
    dataset_config: DecompiledCodeDatasetConfig = DecompiledCodeDatasetConfig(),
    generation_mode: DatasetGenerationMode = "temp",
) -> DecompiledCodeDataset:
    return compile_dataset_task(
        path,
        bins,
        build_command,
        manage_config=manage_config,
        extract_config=extract_config,
        dataset_config=dataset_config,
        generation_mode=generation_mode,
    )
