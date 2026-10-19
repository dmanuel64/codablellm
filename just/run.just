import "globals.just"

[default]
cli *args:
    {{ cargo }} run {{ args }}

python *args: _setup_python_venv
    {{ uv }} --directory {{ quote(pyo3_path) }} run maturin develop
    {{ uv }} --directory {{ quote(pyo3_path) }} run python {{ args }}
