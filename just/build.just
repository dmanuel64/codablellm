import "globals.just"

arch := arch()

[arg("install", long="install", value="true")]
python install="true":
    {{ uv }} run maturin build
    {{ uv }} {{ if install == "true" { "run maturin develop" } else { "-V" } }}

[arg("target", pattern="ubuntu|alpine|windows")]
[group("docker")]
builder target version="latest":
    {{ docker }} buildx build --target {{ target }} \
        -f {{ justfile_directory() / "docker" / if target == "windows" { "builder.windows.Dockerfile" } else { "builder.linux.Dockerfile" } }} \
        -t dmanuel99/codablellm-builder:{{ version }}-{{ target }} \
        --load .

[arg("decompiler", pattern="ghidra")]
[group("docker")]
decompiler decompiler version="latest":
    {{ docker }} buildx build \
        -f {{ justfile_directory() / "docker" / (decompiler + ".Dockerfile") }} \
        -t dmanuel99/codablellm-{{ decompiler }}:{{ version }} \
        --load .
