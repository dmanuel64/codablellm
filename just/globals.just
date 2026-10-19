[windows]
set shell := ["powershell.exe", "-NoLogo", "-Command"]
set no-cd

cargo := if os_family() == "windows" { "& " + quote(require("cargo")) } else { quote(require("cargo")) }
uv := if os_family() == "windows" { "& " + quote(require("uv")) } else { quote(require("uv")) }

pyo3_path := join(justfile_dir(), "crates", "pyo3")

_setup_python_venv:
    {{ uv }} --directory {{ quote(pyo3_path) }} venv --allow-existing
