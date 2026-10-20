import "globals.just"

# Run the CodableLLM CLI
[arg("args", help="Additional arguments to pass to 'cargo run'")]
[default]
cli *args:
    {{ cargo }} run {{ args }}

# Run CodableLLM in a Python REPL
[arg("args", help="Additional arguments to pass to 'python'")]
python *args: _setup_python_venv
    {{ uv }} --directory {{ quote(pyo3_path) }} run maturin develop
    {{ uv }} --directory {{ quote(pyo3_path) }} run python {{ args }}
