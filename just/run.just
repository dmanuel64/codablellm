import "globals.just"

alias codablellm := cli

# Run the CodableLLM CLI
[arg("args", help="Additional arguments to pass to 'cargo run'. Specifying --release will build a release binary with Goreleaser, and the remaining 'args' will be forwarded to the built binary")]
[default]
cli *args:
    {{ if args =~ '--release' { goreleaser + " build --clean --snapshot --single-target " } else { cargo + " run " + args } }}
    {{ if args =~ '--release' { codablellm_release_bin + " " + replace(args, "--release", "") } else { "" } }}

# Run CodableLLM in a Python REPL
[arg("args", help="Additional arguments to pass to 'python'")]
python *args: _setup_python_venv
    {{ cargo }} run --package codablellm-pyo3 --bin stub_gen
    {{ uv }} --directory {{ quote(pyo3_path) }} run maturin develop
    {{ uv }} --directory {{ quote(pyo3_path) }} run python {{ args }}
