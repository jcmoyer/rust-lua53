# rust-lua53 [![Build Status](https://travis-ci.org/jcmoyer/rust-lua53.svg?branch=master)](https://travis-ci.org/jcmoyer/rust-lua53)
Aims to be complete Rust bindings for Lua 5.3 and beyond. Currently, `master` is tracking Lua `5.3.0-beta`.

Requires `gcc` and a copy of Lua 5.3 to build. To get started:

    export LUA_INCLUDE=/path/to/lua53/headers
    ./prebuild.sh
    cargo build

# License
Licensed under the MIT License, which is the same license Lua is distributed
under. Refer to `LICENSE.md` for more information.

