# https://just.systems

cargo := if os_family() == "windows" { "& " + quote(require("cargo")) } else { quote(require("cargo")) }
uv := if os_family() == "windows" { "& " + quote(require("uv")) } else { quote(require("uv")) }
docker := if os_family() == "windows" { "& " + quote(require("docker")) } else { quote(require("docker")) }

[windows]
set shell := ["powershell.exe", "-NoLogo", "-Command"]

default: run

run *args:
    {{ cargo }} run -- {{ args }}

test *args:
    {{ cargo }} test -- {{ args }}

[arg("install", long="install", value="true")]
build-python install="true":
    {{ uv }} run maturin build
    {{ uv }} {{ if install == "true" { "run maturin develop" } else { "-V" } }}

[arg("target", pattern="ubuntu|alpine|windows")]
build-builder target version="latest":
    {{ docker }} buildx build --target {{ target }} \
        -f {{ justfile_directory() / "docker" / if target == "windows" { "builder.windows.Dockerfile" } else { "builder.linux.Dockerfile" } }} \
        -t dmanuel99/codablellm-builder:{{ target }}-{{ version }} \
        --load .

[arg("decompiler", pattern="ghidra")]
build-decompiler decompiler version="latest":
    {{ docker }} buildx build \
        -f {{ justfile_directory() / "docker" / (decompiler + ".Dockerfile") }} \
        -t dmanuel99/codablellm-builder:{{ decompiler }}-{{ version }} \
        --load .
