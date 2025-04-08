"""
codablellm is a framework for creating and curating high-quality code datasets tailored for large language models
"""

import codablellm.logging
from codablellm.core import DecompileConfig, ExtractConfig, decompiler, extractor
from codablellm.dataset import DecompiledCodeDatasetConfig, SourceCodeDatasetConfig
from codablellm.repoman import (
    ManageConfig,
    compile_dataset,
    create_decompiled_dataset,
    create_source_dataset,
)

__version__ = "1.1.1"
__all__ = [
    "create_source_dataset",
    "create_decompiled_dataset",
    "compile_dataset",
    "extractor",
    "decompiler",
    "ExtractConfig",
    "DecompileConfig",
    "ManageConfig",
    "SourceCodeDatasetConfig",
    "DecompiledCodeDatasetConfig",
]

# Configure logger
logger = codablellm.logging.setup_logger()
