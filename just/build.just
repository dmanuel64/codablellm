import "globals.just"

# Build all targets (not including documentation)
[arg("args", help="Additional arguments to pass to 'cargo build'")]
all *args: (_all_rust args) python

_all_rust *args:
    {{ cargo }} build --workspace {{ args }}

# Build the CodableLLM CLI
[arg("args", help="Additional arguments to pass to 'cargo build'")]
[default]
cli *args:
    {{ cargo }} build {{ args }}

# Build the CodableLLM core library
[arg("args", help="Additional arguments to pass to 'cargo build'")]
core *args:
    {{ cargo }} build --package codablellm-core {{ args }}

# Build the CodableLLM Python bindings
[arg("args", help="Additional arguments to pass to 'maturin build'")]
python *args: _setup_python_venv
    {{ uv }} --directory {{ quote(pyo3_path) }} run maturin build {{ args }}

# Build the CodableLLM MCP server
[arg("args", help="Additional arguments to pass to 'cargo build'")]
mcp *args:
    {{ cargo }} build --package codablellm-mcp {{ args }}

# Build Docs.rs documentation
[group("docs")]
rust-docs:
    {{ cargo }} +nightly doc --workspace --no-deps --all-features
