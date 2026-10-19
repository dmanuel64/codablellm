import "globals.just"

export RUSTDOCFLAGS := "--cfg docsrs"

rust-docs:
    {{ cargo }} +nightly doc --workspace --no-deps --all-features --open
