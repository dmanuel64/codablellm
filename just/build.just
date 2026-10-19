import "globals.just"

[default]
cli *args:
    {{ cargo }} build {{ args }}

core *args:
    {{ cargo }} build --package codablellm-core {{ args }}

python *args: _setup_python_venv
    {{ uv }} --directory {{ quote(pyo3_path) }} run maturin build {{ args }}
