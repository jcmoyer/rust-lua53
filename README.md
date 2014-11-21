# rust-lua53 [![Build Status](https://travis-ci.org/jcmoyer/rust-lua53.svg?branch=master)](https://travis-ci.org/jcmoyer/rust-lua53)
Aims to be complete Rust bindings for Lua 5.3 and beyond. Currently, `master` is tracking Lua `5.3.0-beta`.

Requires:
- gcc
- bash
- a Lua 5.3 installation

## Linux

    cargo build

The build script will look for Lua 5.3 headers in `/usr/local/include`. If you
need to specify a different directory, you can export `LUA_INCLUDE`.

## Windows

The cargo situation on Windows isn't so great, so you'll probably get an error
just running `cargo build`. You will have to invoke `prebuild.sh` manually:

    ./prebuild.sh
    cargo build

Like above, if Lua 5.3 headers aren't in `/usr/local/include` (as they should
be if you're using an MSYS2 bash shell), you will need to specify a different
directory using the the environment variable `LUA_INCLUDE`.

# License
Licensed under the MIT License, which is the same license Lua is distributed
under. Refer to `LICENSE.md` for more information.

