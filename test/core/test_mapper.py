from pathlib import Path
from codablellm.core import *
from codablellm.core.mapper import default_mapper


def test_mapper(tmp_path: Path, create_compiled_functions_factory) -> None:
    c_file = tmp_path / "test.c"
    c_function = "int test() { return 0; }"
    c_bin = tmp_path / "c.exe"
    rs_file = tmp_path / "test.rs"
    rs_function = "fn test() -> i32 { return 0; }"
    rs_bin = tmp_path / "rs.exe"

    c_function, decompiled_c_function = create_compiled_functions_factory(
        c_file, c_bin, "C", c_function, "test"
    )
    rs_function, decompiled_rs_function = create_compiled_functions_factory(
        rs_file, rs_bin, "Rust", rs_function, "test"
    )

    assert default_mapper(decompiled_c_function, c_function)
    assert default_mapper(decompiled_rs_function, rs_function)
