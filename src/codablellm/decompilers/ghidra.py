"""
Support for the Ghidra decompiler.
"""

import json
import logging
import os
import subprocess
from pathlib import Path
from tempfile import NamedTemporaryFile, TemporaryDirectory
from typing import Final, List, Optional, Sequence

import psutil

from codablellm.core import utils
from codablellm.core.decompiler import Decompiler
from codablellm.core.function import DecompiledFunction, DecompiledFunctionJSONObject
from codablellm.core.utils import PathLike, is_binary

logger = logging.getLogger(__name__)

# TODO: handle halt_baddata();
# TODO: Ghidra Bridge instead of running subprocess? https://github.com/justfoxing/ghidra_bridge

class Ghidra(Decompiler):
    """
    The Ghidra decompiler.

    This class provides an interface to the Ghidra decompiler, allowing for automated
    decompilation of binaries using Ghidra's `analyzeHeadless` command. It requires
    the `GHIDRA_HEADLESS` environment variable to be set, pointing to the `analyzeHeadless`
    executable.
    """

    ENVIRON_KEY: Final[str] = "GHIDRA_HEADLESS"
    """
    The system environment variable key that must contain the path to Ghidra's 
    `analyzeHeadless` command.
    """

    DEFAULT_DECOMPILE_SCRIPT: Final[Path] = (
        Path(__file__).parent.parent / "resources" / "ghidra_scripts" / "decompile.py"
    )
    """
    The path to the Ghidra decompiler script (`decompile.py`) used during the decompilation process.
    """

    _decompile_script = DEFAULT_DECOMPILE_SCRIPT

    def __init__(self) -> None:
        """
        Initializes a new `Ghidra` decompiler instance.

        Raises:
            ValueError: If GHIDRA_HEADLESS is not set.
        """
        super().__init__()
        ghidra_path = Ghidra.get_path()
        if not ghidra_path:
            raise ValueError(
                f"{Ghidra.ENVIRON_KEY} is not set to Ghidra's analyzeHeadless command"
            )
        self._ghidra_path = ghidra_path

    def decompile(self, path: PathLike) -> Sequence[DecompiledFunction]:
        path = Path(path)
        if not is_binary(path):
            raise ValueError("path must be an existing binary.")
        # Create a temporary directory for the Ghidra project
        with TemporaryDirectory() as project_dir:
            logger.debug(f"Ghidra project directory created at {project_dir}")
            # Create a temporary file to store the JSON output of the decompiled functions
            with NamedTemporaryFile(
                mode="w+", suffix=".json", delete=False
            ) as output_file:
                logger.debug(
                    "Ghidra decompiled functions file created " f"at {output_file.name}"
                )
                output_path = Path(output_file.name)
                try:
                    output_file.close()
                    # Run decompile script
                    try:
                        utils.execute_command(
                            [
                                str(self._ghidra_path),
                                project_dir,
                                "codablellm",
                                "-import",
                                str(path),
                                "-scriptPath",
                                str(Ghidra.get_decompile_script().parent),
                                "-postScript",
                                Ghidra.get_decompile_script().name,
                                str(output_path),
                                "-deleteProject",
                            ],
                            task=f"Decompiling {path.name}...",
                        )
                    except subprocess.CalledProcessError as e:
                        cmd_str = " ".join(e.cmd)
                        raise ValueError(
                            "Ghidra command failed: " f'"{cmd_str}"'
                        ) from e
                    # Deserialize decompiled functions
                    try:
                        json_objects: List[DecompiledFunctionJSONObject] = json.loads(
                            output_path.read_text()
                        )
                    except json.JSONDecodeError as e:
                        raise ValueError(
                            "Could not deserialize decompiled Ghidra functions"
                        ) from e
                    else:
                        return [
                            DecompiledFunction.from_decompiled_json(j)
                            for j in json_objects
                        ]
                finally:
                    Ghidra.reap_zombies(os.getpid())
                    output_path.unlink(missing_ok=True)

    def get_stripped_function_name(self, address: int) -> str:
        return f"FUN_{address:X}"

    @staticmethod
    def set_path(path: PathLike) -> None:
        """
        Set's the path to Ghidra's `analyzeHeadless` command.

        Parameters:
            path: The absolute path to Ghidra's `analyzeHeadless` command.
        """
        os.environ[Ghidra.ENVIRON_KEY] = str(path)
        logger.info(f'Set {Ghidra.ENVIRON_KEY}="{path}"')

    @staticmethod
    def get_path() -> Optional[Path]:
        """
        Retrieves the path to Ghidra's `analyzeHeadless` command.

        Returns:
            The path to Ghidra's `analyzeHeadless` command as a `Path` object, or `None` if the environment variable is not set.
        """
        value = os.environ.get(Ghidra.ENVIRON_KEY)
        return Path(value) if value else None

    @staticmethod
    def set_decompile_script(path: PathLike) -> None:
        """
        Set's the path to Ghidra's `analyzeHeadless` command.

        Parameters:
            path: The absolute path to Ghidra's `analyzeHeadless` command.
        """
        Ghidra._decompile_script = Path(path)

    @staticmethod
    def get_decompile_script() -> Path:
        """
        Retrieves the path to Ghidra's `analyzeHeadless` command.

        Returns:
            The path to Ghidra's `analyzeHeadless` command as a `Path` object, or `None` if the environment variable is not set.
        """
        return Ghidra._decompile_script

    @staticmethod
    def reap_zombies(pid: int):
        for proc in psutil.process_iter(["pid", "ppid", "status", "name"]):
            if proc.info["status"] == psutil.STATUS_ZOMBIE and proc.info["ppid"] == pid:
                try:
                    os.waitpid(proc.info["pid"], 0)
                    logger.debug(f"Reaped PID {proc.info['pid']} ({proc.info['name']})")
                except ChildProcessError:
                    pass
                except Exception as e:
                    logger.error(f"Error while reaping PID {proc.info['pid']}: {e}")
