extern crate gcc;

use std::fs;
use std::io;
use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::ffi::OsString;

trait CommandExt {
    fn execute(&mut self) -> io::Result<()>;
}

impl CommandExt for Command {
    /// Execute the command and return an error if it exited with a failure status.
    fn execute(&mut self) -> io::Result<()> {
        let status = try!(self.status());
        if status.success() {
            Ok(())
        } else {
            Err(io::Error::new(io::ErrorKind::Other, format!("The command\n\
            \t{:?}\n\
            did not run successfully.", self)))
        }
    }
}

/// The command to build lua, with switches for different OSes.
fn build_lua(tooling: &gcc::Tool, source: &Path, build: &Path) -> io::Result<()> {
    // calculate the Lua platform name
    let platform = match env::var("TARGET").unwrap().split('-').nth(2).unwrap() {
        "windows" => "mingw",
        "macos" => "macosx",
        "linux" => "linux",
        "freebsd" => "freebsd",
        "dragonfly" => "bsd",
        // fall back to the "generic" system
        _ => "generic",
    };

    // extract CC and MYCFLAGS from the detected tooling
    let cc = tooling.path();
    let mut cflags = OsString::new();
    for arg in tooling.args() {
        cflags.push(arg);
        cflags.push(" ");
    }

    // VPATH is used to invoke "make" from the directory where we want Lua to
    // be built into, but read the sources from the provided source directory.
    // Setting MAKE to match the command we invoke means that the VPATH and
    // Makefile path will be carried over when the Makefile invokes itself.
    let makefile = source.join("Makefile");
    let make = OsString::from(format!("make -e -f {:?}", makefile.to_string_lossy().replace("\\", "/")));

    // call the makefile
    let mut command = Command::new("make");
    for &(ref key, ref val) in tooling.env() {
        command.env(key, val);
    }
    command.current_dir(build)
        .env("VPATH", source.to_string_lossy().replace("\\", "/"))
        .env("MAKE", make)
        .env("CC", cc)
        .env("MYCFLAGS", cflags)
        .arg("-e")
        .arg("-f").arg(makefile)
        .arg(platform)
        .execute()
}

/// If a static Lua is not yet available from a prior run of this script, this
/// will download Lua and build it. The cargo configuration text to link
/// statically against lua.a is then printed to stdout.
fn prebuild() -> io::Result<()> {
    let lua_dir = match env::var_os("LUA_LOCAL_SOURCE") {
        // If LUA_LOCAL_SOURCE is set, use it
        Some(dir) => PathBuf::from(dir),
        // Otherwise, pull from lua-source/src in the crate root
        None => {
            let mut dir = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").unwrap());
            dir.push("lua-source/src");
            dir
        }
    };
    let build_dir = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    let mut config = gcc::Config::new();

    println!("cargo:rustc-link-lib=static=lua");
    if lua_dir.join("liblua.a").exists() {
        // If liblua.a is already in lua_dir, use it
        println!("cargo:rustc-link-search=native={}", lua_dir.display());
    } else {
        // Otherwise, build from lua_dir into build_dir
        if !build_dir.join("liblua.a").exists() {
            let tooling = config.get_compiler();
            try!(fs::create_dir_all(&build_dir));
            try!(build_lua(&tooling, &lua_dir, &build_dir));
        }
        println!("cargo:rustc-link-search=native={}", build_dir.display());
    }

    // Ensure the presence of glue.rs
    if !build_dir.join("glue.rs").exists() {
        // Compile and run glue.c
        let glue = build_dir.join("glue");
        try!(config.include(&lua_dir).get_compiler().to_command()
            .arg("-I").arg(&lua_dir)
            .arg("src/glue/glue.c")
            .arg("-o").arg(&glue)
            .execute());
        try!(Command::new(glue)
            .arg(build_dir.join("glue.rs"))
            .execute());
    }

    Ok(())
}

fn main() {
    match prebuild() {
        Err(e) => panic!("Error: {}", e),
        Ok(()) => (),
    }
}
