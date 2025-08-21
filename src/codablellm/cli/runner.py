import shlex
from codablellm.cli.config import CLIConfig
import logging
import shutil
from subprocess import CalledProcessError
import sys
from pathlib import Path
from tempfile import TemporaryDirectory
from typing import Final

import yaml

import codablellm
from codablellm.core import decompiler, downloader, utils
from codablellm.core.extractor import ExtractConfig
from codablellm.dataset import DecompiledCodeDatasetConfig, SourceCodeDatasetConfig
from codablellm.repoman import ManageConfig

REPOSITORY: Final[str] = "dmanuel99/codablellm"
TAG: Final[str] = codablellm.__version__
WORKDIR = "/workspace"
COMPOSE_FILE: Final[Path] = (
    Path(__file__).parent.parent / "resources" / "docker-compose.yml"
)

logger = logging.getLogger(__name__)


def _run_containerized(config: CLIConfig) -> None:
    actual_save_as = None
    # Get all non-containerize arguments
    args = [arg for arg in sys.argv[1:] if arg != "--containerize" and arg != "-C"]
    with open(COMPOSE_FILE, "r") as compose_file:
        compose_contents = yaml.safe_load(compose_file)
    with TemporaryDirectory() as temp_dir:
        # For any path arguments, rebase them to the temporary directory
        rebased_paths = {
            path: Path(temp_dir) / f"path_{idx}_{path.name}"
            for idx, path in enumerate(config.get_paths())
        }
        for idx, arg in enumerate(args[::]):
            for path, rebased_path in rebased_paths.items():
                if str(path) in arg:
                    logger.debug(f'Rebasing arg/opt "{path}" to "{rebased_path}"')
                    if path.exists():
                        # TODO: this won't work if there are additional libraries being used in the transform file
                        shutil.copy(path, rebased_path)
                    if path == config.save_as:
                        actual_save_as = rebased_path
                        actual_save_as.touch()
                    args[idx] = args[idx].replace(str(path), rebased_path.name)
        # Set tag in compose file
        compose_file_path = Path(temp_dir) / "docker-compose.yml"
        image, tag = compose_contents["services"]["app"]["image"].split(":")
        tag = TAG
        compose_contents["services"]["app"]["image"] = f"{image}:{tag}"
        with open(compose_file_path, "w") as compose_file:
            yaml.safe_dump(compose_contents, compose_file)
        logger.debug(f'Updated app service image to "{image}:{tag}"')
        try:
            utils.execute_command(
                ["docker", "compose", "run", "--rm", "app", "codablellm", *args],
                task="Running CodableLLM Docker compose file...",
                cwd=temp_dir,
                output_handler="show",
                show_spinner=False,
            )
        except CalledProcessError:
            logger.warning('Retrying docker compose command with with "--no-deps"')
            utils.execute_command(
                [
                    "docker",
                    "compose",
                    "run",
                    "--rm",
                    "--no-deps",
                    "app",
                    "codablellm",
                    *args,
                ],
                task="Running CodableLLM Docker compose file...",
                cwd=temp_dir,
                error_handler="ignore",
                output_handler="show",
                show_spinner=False,
            )
        if actual_save_as:
            # Copy output dataset to actual dataset file
            config.save_as.parent.mkdir(parents=True, exist_ok=True)
            shutil.copy(actual_save_as, config.save_as)


def run(cli_config: CLIConfig) -> None:
    if cli_config.containerize:
        _run_containerized(cli_config)
    else:
        if cli_config.url:
            # Download remote repository and save as local repo path
            if cli_config.git:
                downloader.clone(cli_config.url, cli_config.repo)
            else:
                downloader.decompress(cli_config.url, cli_config.repo)
        # Create the extractor configuration
        extract_config = ExtractConfig(
            transform=cli_config.transform,
            strict=cli_config.strict,
        )
        if cli_config.decompile:
            # Configure the decompiler
            decompiler.set(cli_config.decompiler)
            # Created decompiled dataset config
            dataset_config = DecompiledCodeDatasetConfig(
                extract_config=extract_config,
                decompiler_config=decompiler.DecompileConfig(
                    symbol_remover=cli_config.symbol_remover,
                    recursive=cli_config.recursive,
                    strict=cli_config.strict,
                ),
                mapper=cli_config.mapper,
            )
            if not cli_config.build:
                # Create a mapped source code to decompiled code dataset from an already compiled binary
                dataset = codablellm.create_decompiled_dataset(
                    cli_config.repo,
                    cli_config.bins,
                    extract_config=extract_config,
                    dataset_config=dataset_config,
                )
            else:
                # Build a repository, then create a mapped source to decompiled code dataset
                manage_config = ManageConfig(
                    cleanup_command=cli_config.cleanup,
                    run_from=cli_config.run_from,
                    build_error_handling=cli_config.build_error_handling,
                    cleanup_error_handling=cli_config.cleanup_error_handling,
                    extra_paths=cli_config.extra_path,
                )
        else:
            # Create a source code only dataset
            dataset_config = SourceCodeDatasetConfig(
                generation_mode=cli_config.generation_mode,
                extract_config=extract_config,
            )
            dataset = codablellm.create_source_dataset(
                cli_config.repo, config=dataset_config
            )
        # Save dataset
        dataset.save_as(cli_config.save_as)
