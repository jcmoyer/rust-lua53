# rust-lua53 [![Build Status](https://travis-ci.org/jcmoyer/rust-lua53.svg?branch=master)](https://travis-ci.org/jcmoyer/rust-lua53)
Aims to be complete Rust bindings for Lua 5.3 and beyond. Currently, `master`
is tracking Lua `5.3.0`.

Requires:
- gcc
- bash
- a Lua 5.3 installation

**NOTE:** The build process requires Lua 5.3 headers for code generation
purposes. If you built and installed normally, these will be located in
`/usr/local/include` in both Linux and Windows MSYS2 environments. In this
case, the build script will find them automatically. If for some reason these
headers are elsewhere, you will need to export an environment variable called
`LUA_INCLUDE` that points to a directory containing these
files.

### Using crates.io

Add this to your `Cargo.toml`:

```
[dependencies]
lua = "*"
```

### Using git

Add this to your `Cargo.toml`:

```
[dependencies.lua]
git = "https://github.com/jcmoyer/rust-lua53"
```

# Example

```rust
extern crate lua;

fn main() {
  let mut state = lua::State::new();
  state.open_libs();
  state.do_string("print('hello world!')");
}
```

# License
Licensed under the MIT License, which is the same license Lua is distributed
under. Refer to `LICENSE.md` for more information.
