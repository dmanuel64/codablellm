# https://just.systems

mod build "just/build.just"
mod run "just/run.just"
import "just/globals.just"

test *args:
    {{ cargo }} test {{ args }}
