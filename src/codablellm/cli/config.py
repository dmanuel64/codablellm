from dataclasses import dataclass, fields
import itertools
from pathlib import Path
from typing import List, Literal, Optional

from rich import print
from typer import BadParameter

from codablellm.core.utils import Command, CommandErrorHandler, DynamicSymbol
from codablellm.dataset import DatasetGenerationMode


@dataclass
class CLIConfig:
    repo: Path
    save_as: Path
    bins: Optional[List[Path]]
    build: Optional[Command]
    build_error_handling: CommandErrorHandler
    cleanup: Optional[Command]
    cleanup_error_handling: CommandErrorHandler
    clear_extractors: bool
    containerize: bool
    decompile: bool
    decompiler: DynamicSymbol
    debug: bool
    extra_path: List[Path]
    generation_mode: DatasetGenerationMode
    ghidra: Optional[Path]
    ghidra_script: Path
    git: bool
    mapper: DynamicSymbol
    max_workers: Optional[int]
    # parallel: parameters.ParallelOpt = False,
    recursive: bool
    register_extractor: Optional[List[DynamicSymbol]]
    run_from: Literal["repo", "cwd"]
    strict: bool
    symbol_remover: Literal["strip", "pseudo-strip"]
    transform: Optional[DynamicSymbol]
    url: Optional[str]
    verbose: bool

    def __post_init__(self) -> None:
        # Check for any complex parameter conflicts
        if self.build and not self.decompile:
            print(
                "--build implies --decompile. --decompile will be enabled automatically."
            )
            self.decompile = True
        # Verify one binary was specified if decompiling
        if self.decompile and not self.bins:
            raise BadParameter(
                "Must specify at least one binary for decompiled code datasets.",
                param_hint="bins",
            )

    def get_paths(self) -> List[Path]:
        paths = []
        for maybe_path in itertools.chain.from_iterable(
            [
                [value] if not isinstance(value, list) else value
                for value in [getattr(self, field.name) for field in fields(self)]
                if isinstance(value, (Path, DynamicSymbol, list))
            ]
        ):
            paths.append(
                maybe_path if isinstance(maybe_path, Path) else maybe_path.path
            )
        return paths

    @classmethod
    def from_locals(cls, **kwargs) -> "CLIConfig":
        # Collect field names
        field_names = {field.name for field in fields(CLIConfig)}
        final_kwargs = {}
        for key, value in kwargs.items():
            # Skip over variables that start with _
            if not key.startswith("_"):
                if key not in field_names:
                    raise NotImplementedError(
                        f'CLI config does not have a "{key}" field'
                    )
                final_kwargs[key] = value
        # Verify no fields are missing
        missing_fields = final_kwargs.keys() - field_names
        for missing_field in missing_fields:
            raise NotImplementedError(
                f'CLI config expects a "{missing_field}" variable'
            )
        return cls(**final_kwargs)  # type: ignore
