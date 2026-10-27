import "globals.just"

# Build and serve up locally Docs.rs documentation
rust-docs:
    {{ cargo }} +nightly doc --workspace --no-deps --all-features --open --exclude xtask
