import logging
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


def run_containerized() -> None:
    args = [arg for arg in sys.argv[1:] if arg != "--containerize" or arg != "-C"]
    with open(COMPOSE_FILE, "r") as compose_file:
        compose_contents = yaml.safe_load(compose_file)
    with TemporaryDirectory() as temp_dir:
        # Set tag in compose file
        compose_file_path = Path(temp_dir) / "docker-compose.yml"
        image, tag = compose_contents["services"]["app"]["image"].split(":")
        tag = TAG
        compose_contents["services"]["app"]["image"] = f"{image}:{tag}"
        with open(compose_file_path, "w") as compose_file:
            yaml.safe_dump(compose_contents, compose_file)
        logger.debug(f'Updated app service image to "{image}:{tag}"')
        utils.execute_command(
            ["docker", "compose", "run", "--rm", "app", "codablellm", *args],
            task="Running CodableLLM Docker compose file...",
            cwd=temp_dir,
        )
