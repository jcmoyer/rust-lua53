#!/bin/sh
echo "Making directory build/..."
mkdir -p build/

if [ -z "$LUA_INCLUDE" ]; then
  echo "The variable LUA_INCLUDE is not set. Ensure that it points to a directory"
  echo "containing Lua 5.3 header files and re-run this script."
  exit 1
fi

if [ ! -f "$LUA_INCLUDE/lua.h" ]; then
  echo "Could not find file $LUA_INCLUDE/lua.h"
  echo "Ensure that the variable LUA_INCLUDE points to a directory containing Lua 5.3"
  echo "header files."
  exit 1
fi

echo "Compiling glue.c..."
gcc -I$LUA_INCLUDE "src/glue/glue.c" -o build/glue

if [ $? -ne 0 ]; then
  echo "Compilation error; aborting."
  exit 1
fi

echo "Generating glue.rs..."
build/glue "src/ffi/glue.rs"

if [ $? -eq 0 ]; then
  echo "OK! Code generation successful!"
fi

