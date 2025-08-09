import logging
import shutil
from subprocess import CalledProcessError
import sys
from pathlib import Path
from tempfile import TemporaryDirectory
from typing import Final

import yaml

import codablellm
from codablellm.core import utils

REPOSITORY: Final[str] = "dmanuel99/codablellm"
TAG: Final[str] = codablellm.__version__
WORKDIR = "/workspace"
COMPOSE_FILE: Final[Path] = Path(__file__).parent / "resources" / "docker-compose.yml"

logger = logging.getLogger(__name__)


def run_containerized(save_as: Path, *other_paths: Path) -> None:
    # TODO: implement other paths in caller
    actual_save_as = None
    # Get all non-containerize arguments
    args = [arg for arg in sys.argv[1:] if arg != "--containerize" and arg != "-C"]
    with open(COMPOSE_FILE, "r") as compose_file:
        compose_contents = yaml.safe_load(compose_file)
    with TemporaryDirectory() as temp_dir:
        # For any path arguments, copy them to the temporary directory
        for idx, arg in enumerate(args[::]):
            try:
                path = Path(arg)
                if path in other_paths or path == save_as:
                    rebased_path = Path(temp_dir) / f"arg_{idx}_{path.name}"
                    logger.debug(f"Rebasing arg/opt '{path}' to '{rebased_path}'")
                    if path.exists():
                        # TODO: this won't work if there are additional libraries being used in the transform file
                        shutil.copy(path, rebased_path)
                    if path == save_as:
                        actual_save_as = rebased_path
                        actual_save_as.touch()
                    args[idx] = str(rebased_path)
            except Exception:
                pass
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
            save_as.parent.mkdir(parents=True, exist_ok=True)
            shutil.copy(actual_save_as, save_as)
