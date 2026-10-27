import "globals.just"

# Build all targets (not including documentation)
[arg("args", help="Additional arguments to pass to 'cargo build'")]
all *args: (_all_rust args) python

_all_rust *args:
    {{ cargo }} build --workspace {{ args }}

alias codablellm := cli

# Build the CodableLLM CLI
[arg("args", help="Additional arguments to pass to 'cargo build'. Specifying --release will build a release binary with Goreleaser, and the remaining 'args' will be omitted")]
[default]
cli *args:
    {{ if args =~ '--release' { goreleaser + " build --clean --snapshot --single-target " } else { cargo + " build " + args } }}

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
    {{ cargo }} +nightly doc --workspace --no-deps --all-features --exclude xtask

# Build man ages for UNIX systems
[group("docs")]
man-pages:
    {{ cargo }} xtask man

# Build shell auto-completions for all compatible shells
[group("docs")]
completions:
    {{ cargo }} xtask completions
