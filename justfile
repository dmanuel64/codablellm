# https://just.systems
set default-list

mod build "just/build.just"
mod run "just/run.just"
mod serve "just/serve.just"

import "just/globals.just"

[windows]
clean *args:
    {{ cargo }} clean {{ args }}
    -Remove-Item -Path {{ quote(join(pyo3_path, ".venv")) }} -Recurse -Force -ErrorAction SilentlyContinue

[linux]
clean *args:
    {{ cargo }} clean {{ args }}
    rm -rf {{ quote(pyo3_path / ".venv") }}

test *args:
    {{ cargo }} test {{ args }}
