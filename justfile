# https://just.systems
set default-list

# Build commands
mod build "just/build.just"
# Run commands
mod run "just/run.just"
# Build and serve up local documentation
mod serve "just/serve.just"

import "just/globals.just"

# Remove all built artifacts
[arg("args", help="Additional arguments to pass to 'cargo clean'")]
[windows]
clean *args:
    {{ cargo }} clean {{ args }}
    -Remove-Item -Path {{ quote(join(pyo3_path, ".venv")) }} -Recurse -Force -ErrorAction SilentlyContinue

# Remove all built artifacts
[arg("args", help="Additional arguments to pass to 'cargo clean'")]
[linux]
clean *args:
    {{ cargo }} clean {{ args }}
    rm -rf {{ quote(pyo3_path / ".venv") }}

# Run cargo tests
[arg("args", help="Additional arguments to pass to 'cargo test'")]
test *args:
    {{ cargo }} test --workspace {{ args }}
