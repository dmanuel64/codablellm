"""
Code dataset generation.
"""

import logging
import os
from abc import ABC, abstractmethod
from collections.abc import Mapping
from dataclasses import dataclass, field, replace
from pathlib import Path
from typing import (
    Any,
    Callable,
    Collection,
    Dict,
    Final,
    Iterable,
    Iterator,
    List,
    Literal,
    NamedTuple,
    Optional,
    TypeVar,
    Union,
)
from typing_extensions import deprecated

from pandas import DataFrame
from prefect import task

from codablellm.core import decompiler, extractor, utils
from codablellm.core.function import DecompiledFunction, SourceFunction
from codablellm.core.mapper import DEFAULT_MAPPER, Mapper

logger = logging.getLogger(__name__)


class Dataset(ABC):
    """
    A code dataset.
    """

    @abstractmethod
    def to_df(self) -> DataFrame:
        """
        Converts the code dataset to a pandas DataFrame.

        Returns:
            A pandas DataFrame representation of the code dataset.
        """
        pass

    def save_as(self, path: utils.PathLike) -> None:
        """
        Converts the dataset to a DataFrame and exports it to the specified file path based on
        its extension. The export format is determined by the file extension provided in the
        `path` parameter.

        Example:
            ```py
            dataset.save_as("output.xlsx")
            ```

            Successfully saves the dataset as an Excel file to "output.xlsx".

        Supported Formats and Extensions:
            - JSON: .json, .jsonl
            - CSV/TSV: .csv, .tsv
            - Excel: .xlsx, .xls, .xlsm **(requires codablellm[excel])**
            - Markdown: .md, .markdown **(requires codablellm[markdown])**
            - LaTeX: .tex
            - HTML: .html, .htm
            - XML: .xml **(requires codablellm[xml])**

        Parameters:
            path: Path to save the dataset at.

        Raises:
            ValueError: If the provided file extension is unsupported.
            ExtraNotInstalled: If the file extension requires an additional library that is not installed.
        """

        @utils.requires_extra("excel", "Excel exports", "openpyxl")
        def to_excel(df: DataFrame, path: Path) -> None:
            df.to_excel(path)

        @utils.requires_extra("xml", "XML exports", "lxml")
        def to_xml(df: DataFrame, path: Path) -> None:
            df.to_xml(path)

        @utils.requires_extra("markdown", "Markdown exports", "tabulate")
        def to_markdown(df: DataFrame, path: Path) -> None:
            df.to_markdown(path)

        path = Path(path)
        extension = path.suffix.casefold()
        if extension in [e.casefold() for e in [".json", ".jsonl"]]:
            self.to_df().to_json(
                path, lines=extension == ".jsonl".casefold(), orient="records"
            )
        elif extension in [e.casefold() for e in [".csv", ".tsv"]]:
            self.to_df().to_csv(
                path, sep="," if extension == ".csv".casefold() else "\t"
            )
        elif extension in [e.casefold() for e in [".xlsx", ".xls", ".xlsm"]]:
            to_excel(self.to_df(), path)
        elif extension in [e.casefold() for e in [".md", ".markdown"]]:
            to_markdown(self.to_df(), path)
        elif extension == ".tex".casefold():
            self.to_df().to_latex(path)
        elif extension in [e.casefold() for e in [".html", ".htm"]]:
            self.to_df().to_html(path)
        elif extension == ".xml".casefold():
            to_xml(self.to_df(), path)
        else:
            raise ValueError(f"Unsupported file extension: {path.suffix}")
        logger.info(f"Successfully saved {path.name}")


DatasetGenerationMode = Literal["path", "temp", "temp-append"]
"""
How the dataset should be generated.

Generation Modes:
    - **`path`**: Generates the dataset directly from the local repository path.
        - *Note*: If `extract_config.transform` is provided, the source code in the local repository 
        may be overridden by the transformed code.

    - **`temp`**: Copies the repository to a temporary directory and generates the dataset there.
        - *If `extract_config.transform` is not provided, the mode defaults to `path`*.
    - **`temp-append`**: Copies the repository to a temporary directory, applies the transformation
    using `extract_config.transform`, and appends the transformed entries to the original source
    code from the local repository.
        - *If `extract_config.transform` is not provided, the mode defaults to `path`*.
"""


# TODO: see if there's a way to make this a frozen dataclass


@dataclass
class SourceCodeDatasetConfig:
    """
    Configuration options for generating a source code dataset.

    This class provides flexible options for controlling how a source code dataset is generated,
    including handling of temporary directories, extraction settings, and generation modes.
    """

    generation_mode: DatasetGenerationMode = "temp"
    """
    How the source code dataset should be generated.
    """
    delete_temp: bool = True
    """
    Controls whether the temporary directory should be deleted after dataset generation.

    - *Applies only if `generation_mode` is set to `temp`. When set to `True`, 
    the temporary directory will be automatically deleted after dataset generation.*
    """
    extract_config: extractor.ExtractConfig = field(
        default_factory=extractor.ExtractConfig
    )
    """
    Configuration settings for extracting source code functions.
    """
    log_generation_warning: bool = True

    def __post_init__(self) -> None:
        if (
            self.generation_mode == "temp" or self.generation_mode == "temp-append"
        ) and not self.extract_config.transform:
            if self.log_generation_warning:
                logger.warning(
                    f'Generation mode was specified as "{self.generation_mode}", but no '
                    'transform was provided. Changing generation mode to "path" to '
                    "save resources"
                )
            self.generation_mode = "path"


T = TypeVar("T")


class SourceCodeDataset(Dataset, Mapping[str, SourceFunction]):
    """
    A source code dataset.

    This class provides functionality to manage and interact with a collection of
    source functions, allowing indexing and mapping by unique identifiers (UIDs)
    """

    def __init__(self, functions: Iterable[SourceFunction]) -> None:
        """
        Initializes a new source code dataset instance with a collection of source functions.

        Parameters:
            functions: An iterable collection of source code functions used to populate the dataset.
        """
        super().__init__()
        self._mapping: Dict[str, SourceFunction] = {f.uid: f for f in functions}

    def __getitem__(self, key: Union[str, SourceFunction]) -> SourceFunction:
        if isinstance(key, SourceFunction):
            return self[key.uid]
        return self._mapping[key]

    def __iter__(self) -> Iterator[str]:
        return iter(self._mapping)

    def __len__(self) -> int:
        return len(self._mapping)

    def get(
        self, key: Union[str, SourceFunction], default: T = None
    ) -> Union[SourceFunction, T]:
        try:
            return self[key]
        except KeyError:
            return default

    def to_df(self) -> DataFrame:
        function_dicts: List[Dict[str, Any]] = []
        for function in self.values():
            function_json = function.to_json()
            function_dict = dict(function_json)
            # Flatten SourceFunction.metadata
            del function_dict["metadata"]
            function_dict.update(function_json["metadata"])
            function_dicts.append(function_dict)
        try:
            return DataFrame(function_dicts).set_index("uid")
        except KeyError:
            logger.debug(
                'Could not set DataFrame index to "uid", returning an empty '
                "DataFrame to assume that the DataFrame is empty"
            )
            return DataFrame()

    def get_common_directory(self) -> Path:
        """
        Returns the common directory shared by all entries in the dataset. This typically
        represents the path to the local repository from which the dataset was generated.

        Returns:
            The common directory path for all dataset entries.
        """
        common_path = Path(os.path.commonpath(p.path for p in self.values()))
        return common_path if common_path.is_dir() else common_path.parent

    @classmethod
    def create_aligned_dataset(
        cls,
        original: Union[Collection[SourceFunction], "SourceCodeDataset"],
        transformed: Union[Collection[SourceFunction], "SourceCodeDataset"],
    ) -> "SourceCodeDataset":
        # Create temporary transformed and non-transformed datasets (if not already)
        if not isinstance(original, SourceCodeDataset):
            original = cls(function for function in original)
        if not isinstance(transformed, SourceCodeDataset):
            transformed = cls(function for function in transformed)
        annotated_functions: List[SourceFunction] = []
        for transformed_function in transformed.values():
            # Check if UID's match in original dataset
            function = original.get(transformed_function)
            if function:
                # Annotate with metadata
                logger.info(f"Annotating {function.uid}...")
                annotated_function = replace(
                    function,
                    _metadata={
                        **function.metadata,
                        "transformed_definition": transformed_function.definition,
                        "transformed_class_name": transformed_function.class_name,
                    },
                )
                annotated_functions.append(annotated_function)
            else:
                logger.warning(f'Could not locate UID "{transformed_function.uid}"')
        return cls(annotated_functions)

    @classmethod
    @utils.codablellm_task(
        name="create_source_dataset", on_completion=[utils.benchmark_task]
    )
    def from_repository(
        cls,
        path: utils.PathLike,
        config: SourceCodeDatasetConfig = SourceCodeDatasetConfig(
            log_generation_warning=False
        ),
    ) -> "SourceCodeDataset":
        """
        Creates a source code dataset from a local repository.

        This method scans the specified repository and generates a dataset of source code functions
        based on the provided configuration. Optionally, it can return a callable pool that allows
        deferred execution of the dataset generation process.

        Example:
            ```py
            SourceCodeDataset.from_repository('path/to/my/repository',
                                                config=SourceCodeDatasetConfig(
                                                    generation_mode='path'
                                                    extract_config=ExtractConfig(
                                                        transform=remove_comments
                                                    )
                                                )
                                             )
            ```

            Will create a source code dataset from `path/to/my/repository`, overriding the contents
            of the repository and removing all comments from the extracted source code functions.

        Parameters:
            path: Path to the local repository to generate the dataset from.
            config: Configuration settings for dataset generation.
            as_callable_pool: If `True`, returns a `CallablePoolProgress` object that can be executed later to generate the dataset.

        Returns:
            The generated source code dataset if `as_callable_pool` is `False`, or a `CallablePoolProgress` object if `as_callable_pool` is `True`.
        """
        original_path = path
        with utils.prepared_dir(
            path,
            rebased=config.generation_mode == "temp"
            or config.generation_mode == "temp-append",
            set_env_var=False,
        ) as path:
            logger.info("Submitting extraction task...")
            # Extract source code functions on the path/temp directory
            futures = extractor.extract_directory_task.submit(
                path, config.extract_config
            )
            if config.generation_mode == "temp-append":
                # Create a copy of the extract config to extract the path without a transform
                no_transform_extract_config = replace(
                    config.extract_config, transform=None
                )
                original_futures = extractor.extract_directory_task.submit(
                    original_path, config=no_transform_extract_config
                )
                return cls.create_aligned_dataset(
                    original_futures.result(), futures.result()
                )
            return cls(function for function in futures.result())


@dataclass(frozen=True)
class DecompiledCodeDatasetConfig:
    """
    Configuration options for generating a decompiled dataset.

    This class defines the settings for extracting source code functions from binaries
    and configuring the decompilation process.
    """

    extract_config: extractor.ExtractConfig = field(
        default_factory=extractor.ExtractConfig
    )
    """
    Configuration settings for extracting source code functions.
    """
    decompiler_config: decompiler.DecompileConfig = field(
        default_factory=decompiler.DecompileConfig
    )
    """
    Configuration settings for decompiling binaries.
    """
    mapper: utils.DynamicSymbol = field(default_factory=lambda: DEFAULT_MAPPER)
    """
    The mapping function used to determine if a decompiled function corresponds to a given source function.
    """

    def get_mapper(self) -> Mapper:
        return utils.dynamic_import(self.mapper)


class MappedFunction(NamedTuple):
    decompiled_function: DecompiledFunction
    source_functions: SourceCodeDataset


class DecompiledCodeDataset(Dataset, Mapping[str, MappedFunction]):
    """
    A dataset of decompiled functions mapped to their corresponding potential source functions.

    This class provides functionality to manage and interact with decompiled functions
    and their possible source code counterparts, allowing for easy lookup by unique identifiers (UIDs).
    """

    def __init__(self, mappings: Iterable[MappedFunction]) -> None:
        """
        Initializes a new decompiled code dataset instance with a collection of mappings
        between decompiled functions and their potential source functions.

        Parameters:
            mappings: An iterable collection of 2-tuples, where each tuple consists of the decompiled function and the corresponding potential source functions.
        """
        super().__init__()
        self._mapping: Dict[str, MappedFunction] = {m[0].uid: m for m in mappings}

    def __getitem__(self, key: Union[str, DecompiledFunction]) -> MappedFunction:
        if isinstance(key, DecompiledFunction):
            return self[key.uid]
        return self._mapping[key]

    def __iter__(self) -> Iterator[str]:
        return iter(self._mapping)

    def __len__(self) -> int:
        return len(self._mapping)

    def get(
        self, key: Union[str, DecompiledFunction], default: T = None
    ) -> Union[MappedFunction, T]:
        try:
            return self[key]
        except KeyError:
            return default

    def to_df(self) -> DataFrame:
        function_dicts: List[Dict[str, Any]] = []
        for decompiled_function, source_functions in self.values():
            decompiled_function_json = decompiled_function.to_json()
            decompiled_function_dict = dict(decompiled_function_json)
            # Flatten DecompiledFunction.metadata
            del decompiled_function_dict["metadata"]
            decompiled_function_dict.update(decompiled_function_json["metadata"])
            # Refactor names to be more specific on decompiled functions and multiple source functions
            decompiled_function_dict["decompiled_uid"] = decompiled_function_dict.pop(
                "uid"
            )
            decompiled_function_dict["bin"] = decompiled_function_dict.pop("path")
            decompiled_function_dict["decompiled_definition"] = (
                decompiled_function_dict.pop("definition")
            )
            source_functions_dict = source_functions.to_df().to_dict()
            source_functions_dict["source_files"] = source_functions_dict.pop("path")
            source_functions_dict["source_definitions"] = source_functions_dict.pop(
                "definition"
            )
            del source_functions_dict["name"]
            source_functions_dict["source_file_start_bytes"] = (
                source_functions_dict.pop("start_byte")
            )
            source_functions_dict["source_file_end_bytes"] = source_functions_dict.pop(
                "end_byte"
            )
            source_functions_dict["class_names"] = source_functions_dict.pop(
                "class_name"
            )
            decompiled_function_dict.update(source_functions_dict)  # type: ignore
            function_dicts.append(decompiled_function_dict)
        try:
            return DataFrame(function_dicts).set_index("decompiled_uid")
        except KeyError:
            logger.debug(
                'Could not set DataFrame index to "uid", returning an empty '
                "DataFrame to assume that the DataFrame is empty"
            )
            return DataFrame()

    def lookup(self, key: Union[str, SourceFunction]) -> List[MappedFunction]:
        """
        Finds all mappings where the given key may correspond to potential source functions.

        The method searches through the dataset and returns all decompiled functions
        and their associated source code datasets where the specified key matches one of the
        source functions.

        Parameters:
            key: The key to search for, which can be either a source function UID or a `SourceFunction` object.

        Returns:
            A list of tuples, where each tuple consists of a decompiled function and its
            corresponding source code dataset containing the potential matches.
        """
        return [m for m in self.values() if key in m[1]]

    def to_source_code_dataset(self) -> SourceCodeDataset:
        """
        Converts the decompiled code dataset into a source code dataset.

        This method aggregates all source functions from the decompiled code dataset
        and constructs a `SourceCodeDataset` containing only the source functions.

        Returns:
            A dataset containing all source functions extracted from the decompiled code dataset.
        """
        return SourceCodeDataset(f for _, d in self.values() for f in d.values())

    @deprecated('Use decompiler.DecompileConfig.symbol_remover = "pseudo-strip"')
    def to_stripped_dataset(self) -> "DecompiledCodeDataset":
        """
        Converts the decompiled code dataset into a stripped decompiled code dataset.

        The method applies the stripping process to each decompiled function in the dataset,
        resulting in a dataset with stripped versions of the decompiled functions.

        Returns:
            A new dataset where all decompiled functions have been stripped.
        """
        return DecompiledCodeDataset(
            MappedFunction(d.to_stripped(), s) for d, s in self.values()
        )

    @classmethod
    @utils.codablellm_task(name="create_decompiled_dataset")
    def from_repository(
        cls,
        path: utils.PathLike,
        bins: Collection[utils.PathLike],
        extract_config: extractor.ExtractConfig = extractor.ExtractConfig(),
        dataset_config: DecompiledCodeDatasetConfig = DecompiledCodeDatasetConfig(),
    ) -> "DecompiledCodeDataset":
        """
        Creates a decompiled code dataset from a built local repository.

        This method scans the specified local repository, decompiles the provided binaries,
        and generates a dataset of decompiled functions mapped to their corresponding potential
        source code functions based on the provided extraction and dataset configuration.

        Example:
            ```py
            DecompiledCodeDataset.from_repository('path/to/my/repository',
                                                [
                                                'path/to/my/repository/bin1.exe',
                                                'path/to/my/repository/bin2.exe'
                                                ],
                                                extract_config=ExtractConfig(
                                                    transform=remove_comments
                                                ),
                                                dataset_config=DecompiledCodeDatasetConfig(
                                                    strip=True
                                                )
                                             )
            ```

            The above example creates a decompiled code dataset from a copy of
            `path/to/my/repository`, removes all comments from the extracted source code
            functions, decompiles the binaries `bin1.exe` and `bin2.exe`, and strips the symbols
            after decompilation.

        Parameters:
            path: Path to the local repository to generate the dataset from.
            bins: A sequence of paths to the built binaries of the repository that should be decompiled.
            extract_config: Configuration settings for extracting source code functions.
            dataset_config: Configuration settings for generating the decompiled code dataset.

        Returns:
            The generated dataset containing mappings of decompiled functions to their potential source code functions.

        Raises:
            ValueError: If `bins` is an empty sequence.
        """
        bins = [bins] if isinstance(bins, str) else bins
        if not any(bins):
            raise ValueError("Must at least specify one binary")
        # Extract source code functions and decompile binaries in parallel
        future_functions = extractor.extract_directory_task.submit(
            path, config=extract_config
        )
        future_bins = [
            decompiler.decompile_bins_task.submit(
                bin, config=dataset_config.decompiler_config
            )
            for bin in bins
        ]
        return cls.map_functions(
            future_functions.result(),
            [f for future in future_bins for f in future.result()],
            config=dataset_config,
        )

    @staticmethod
    def _build_function_name_map(
        source_functions: Iterable[SourceFunction],
    ) -> Dict[str, List[SourceFunction]]:
        fn_map: Dict[str, List[SourceFunction]] = {}
        for source_function in source_functions:
            fn_map.setdefault(
                SourceFunction.get_function_name(source_function.uid), []
            ).append(source_function)
        return fn_map

    # TODO: maybe make into prefect task? Just set max threads
    @staticmethod
    def _map_decompiled_function(
        decompiled_function: DecompiledFunction,
        function_name_map: Dict[str, List[SourceFunction]],
        config: DecompiledCodeDatasetConfig,
    ) -> Optional[MappedFunction]:
        logger.debug(f"Aligning decompiled function: {repr(decompiled_function.name)}")
        try:
            source_candidates = function_name_map.get(decompiled_function.name, [])
            source_functions = [
                s
                for s in source_candidates
                if config.get_mapper()(decompiled_function, s)
            ]
            if not source_functions:
                return None
            return MappedFunction(
                decompiled_function, SourceCodeDataset(source_functions)
            )
        except Exception as e:
            logger.error(
                f"Error aligning function {repr(decompiled_function.name)}: {repr(e)}"
            )
            return None

    @classmethod
    def map_functions(
        cls,
        source: Union[SourceCodeDataset, Iterable[SourceFunction]],
        decompiled: Union["DecompiledCodeDataset", Iterable[DecompiledFunction]],
        config: DecompiledCodeDatasetConfig = DecompiledCodeDatasetConfig(),
    ) -> "DecompiledCodeDataset":

        # Normalize source
        if not isinstance(source, SourceCodeDataset):
            source = SourceCodeDataset(f for f in source)

        # Normalize decompiled
        if isinstance(decompiled, DecompiledCodeDataset):
            decompiled = [m.decompiled_function for m in decompiled.values()]

        logger.info("Building function name map...")
        function_name_map = DecompiledCodeDataset._build_function_name_map(
            source.values()
        )

        logger.info("Mapping decompiled functions to source functions...")

        # Gather results and filter None
        mappings = [
            DecompiledCodeDataset._map_decompiled_function(
                func, function_name_map, config
            )
            for func in decompiled
        ]
        mappings = [m for m in mappings if m]

        logger.info(
            f"Successfully mapped {len(mappings)} decompiled functions to "
            f"{sum(len(f) for f in function_name_map.values())} source functions"
        )
        return DecompiledCodeDataset(mappings)

    @classmethod
    def create_aligned_dataset(
        cls, original: "DecompiledCodeDataset", transformed: "DecompiledCodeDataset"
    ) -> "DecompiledCodeDataset":
        annotated_functions: List[MappedFunction] = []
        for transformed_function, _ in transformed.values():
            # Check if UID's match in original dataset
            decompiled_function, source_functions = original.get(
                transformed_function, (None, None)
            )
            if decompiled_function and source_functions:
                # Annotate with metadata
                logger.info(f"Annotating {decompiled_function.uid}...")
                annotated_function = replace(
                    decompiled_function,
                    _metadata={
                        **decompiled_function.metadata,
                        "transformed_definition": transformed_function.definition,
                        "transformed_assembly": transformed_function.assembly,
                    },
                )
                annotated_functions.append(
                    MappedFunction(annotated_function, source_functions)
                )
            else:
                logger.warning(f'Could not locate UID "{transformed_function.uid}"')
        return cls(annotated_functions)
