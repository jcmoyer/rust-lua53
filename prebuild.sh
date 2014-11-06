#!/bin/sh
mkdir -p build/
gcc -I$LUA_INCLUDE "src/glue/glue.c" -o build/glue
build/glue "src/ffi/glue.rs"

