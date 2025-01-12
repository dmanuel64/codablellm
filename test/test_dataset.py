from pathlib import Path

import pytest
from codablellm.core.extractor import ExtractConfig
from codablellm.dataset import *


def test_save_dataset(tmp_path: Path) -> None:
    empty_dataset = SourceCodeDataset([])
    for ext in ['.json', '.jsonl', '.csv', '.tsv',
                '.xlsx', '.xls', '.xlsm', '.md',
                '.markdown', '.tex', '.html',
                '.html']:
        path = (tmp_path / 'dataset').with_suffix(ext)
        empty_dataset.save_as(path)
    with pytest.raises(ValueError):
        empty_dataset.save_as(tmp_path / 'dataset.unknown')


def test_source_dataset(c_repository: Path) -> None:
    dataset = SourceCodeDataset.from_repository(c_repository,
                                                SourceCodeDatasetConfig(generation_mode='path'))
    temp_dataset = SourceCodeDataset.from_repository(c_repository)
    assert len(dataset) == 8
    assert len(temp_dataset) == 8
    assert dataset.get_common_path() == c_repository
    assert temp_dataset.get_common_path() == c_repository
    assert dataset.get('file1.c::function1') is not None
    functions = dataset.values()
    assert dataset.to_df().to_dict() == DataFrame({'path': [str(f.path) for f in functions],
                                                   'uid': [f.uid for f in functions],
                                                   'class_name': [f.class_name for f in functions],
                                                   'definition': [f.definition for f in functions],
                                                   'start_byte': [f.start_byte for f in functions],
                                                   'end_byte': [f.end_byte for f in functions],
                                                   'language': [f.language for f in functions],
                                                   'name': [f.name for f in functions]}).set_index('uid').to_dict()


def test_modified_source_dataset(c_repository: Path) -> None:
    dataset = SourceCodeDataset.from_repository(c_repository,
                                                SourceCodeDatasetConfig(
                                                    extract_config=ExtractConfig(
                                                        transform=lambda s: s.with_definition(
                                                            '', metadata={'custom_field': True})
                                                    )
                                                )
                                                )
    assert len(dataset) == 8
    functions = dataset.values()
    assert dataset.to_df().to_dict() == DataFrame({'path': [str(f.path) for f in functions],
                                                   'uid': [f.uid for f in functions],
                                                   'class_name': [f.class_name for f in functions],
                                                   'definition': [f.definition for f in functions],
                                                   'start_byte': [f.start_byte for f in functions],
                                                   'end_byte': [f.end_byte for f in functions],
                                                   'language': [f.language for f in functions],
                                                   'custom_field': [f.metadata['custom_field'] for f in functions],
                                                   'name': [f.name for f in functions]}).set_index('uid').to_dict()
    dataset = SourceCodeDataset.from_repository(c_repository,
                                                SourceCodeDatasetConfig(
                                                    generation_mode='temp-append',
                                                    extract_config=ExtractConfig(
                                                        transform=lambda s: s.with_definition(
                                                            '', metadata={'custom_field': False})
                                                    )
                                                )
                                                )
    assert len(dataset) == 8
    functions = dataset.values()
    assert dataset.to_df().to_dict() == DataFrame({'path': [str(f.path) for f in functions],
                                                   'uid': [f.uid for f in functions],
                                                   'class_name': [f.class_name for f in functions],
                                                   'definition': [f.definition for f in functions],
                                                   'start_byte': [f.start_byte for f in functions],
                                                   'end_byte': [f.end_byte for f in functions],
                                                   'language': [f.language for f in functions],
                                                   'transformed_definition': [f.metadata['transformed_definition'] for f in functions],
                                                   'transformed_class_name': [f.metadata['transformed_class_name'] for f in functions],
                                                   'custom_field': [f.metadata['custom_field'] for f in functions],
                                                   'name': [f.name for f in functions]}).set_index('uid').to_dict()


def test_decompiled_dataset(c_repository: Path, c_bin: Path) -> None:
    dataset = DecompiledCodeDataset.from_repository(c_repository, [c_bin])
    assert len(dataset) == 0
