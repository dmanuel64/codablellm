[windows]
set shell := ["powershell.exe", "-NoLogo", "-Command"]
set no-cd

export RUSTDOCFLAGS := "--cfg docsrs"

cargo := if os_family() == "windows" { "& " + quote(require("cargo")) } else { quote(require("cargo")) }
uv := if os_family() == "windows" { "& " + quote(require("uv")) } else { quote(require("uv")) }
goreleaser := if os_family() == "windows" { "& " + quote(require("goreleaser")) } else { quote(require("goreleaser")) }
codablellm_release_bin := if os_family() == "windows" { "& " + quote(release_bin_path) } else { quote(release_bin_path) }

pyo3_path := join(justfile_dir(), "crates", "pyo3")
goreleaser_dist_path := join(justfile_dir(), "dist")
# TODO: Linux is hardcoded as GNU, this won't work for people developing on Alpine
release_bin_path := join(goreleaser_dist_path, if os() == "windows" { "codablellm-windows-native_" + arch() + "-pc-windows-msvc" } else if os() == "darwin" { "codablellm-darwin_darwin_all" } else if os() == "linux" { "codablellm-linux_" + arch() + "-unknown-linux-gnu" } else { error("Unsupported OS: " + os()) }, "codablellm")

_setup_python_venv:
    {{ uv }} --directory {{ quote(pyo3_path) }} venv --allow-existing
