# https://just.systems

mod build "just/build.just"
import "just/globals.just"

default: run

run *args:
    {{ cargo }} run -- {{ args }}

test *args:
    {{ cargo }} test {{ args }}
