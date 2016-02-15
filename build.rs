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
fn build_lua(tooling: &gcc::Tool, dir: &Path) -> io::Result<()> {
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

    // build the CC and MYCFLAGS parameters
    let mut cc = OsString::from("CC=");
    cc.push(tooling.path());
    let mut cflags = OsString::from("MYCFLAGS=");
    for arg in tooling.args() {
        cflags.push(arg);
        cflags.push(" ");
    }

    // call the makefile
    let mut command = Command::new("make");
    for &(ref key, ref val) in tooling.env() {
        command.env(key, val);
    }
    command.current_dir(dir)
        .arg(platform)
        .arg(cc)
        .arg(cflags)
        .execute()
}

/// The command to fetch a URL (e.g. with wget) specialized for different
/// OSes.
#[cfg(not(any(target_os = "freebsd", target_os = "dragonfly", target_os = "macos")))]
fn fetch_in_dir(url: &str, cwd: &Path) -> io::Result<()> {
    Command::new("wget").arg(url).current_dir(cwd).execute()
}

#[cfg(any(target_os = "freebsd", target_os = "dragonfly"))]
fn fetch_in_dir(url: &str, cwd: &Path) -> io::Result<()> {
    Command::new("fetch").arg(url).current_dir(cwd).execute()
}

#[cfg(target_os = "macos")]
fn fetch_in_dir(url: &str, cwd: &Path) -> io::Result<()> {
    Command::new("curl").arg("-O").arg(url).current_dir(cwd).execute()
}

/// If a static Lua is not yet available from a prior run of this script, this
/// will download Lua and build it. The cargo configuration text to link
/// statically against lua.a is then printed to stdout.
fn prebuild() -> io::Result<()> {
    let lua_version = match env::var_os("LUA_VERSION") {
        Some(lua_version) => lua_version,
        None => From::from("5.3.0"),
    };
    let lua_version = lua_version.to_str().unwrap();
    let lua_dir = format!("lua-{}", lua_version);
    let lua_tarball = format!("{}.tar.gz", lua_dir);
    let build_dir = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    let tooling = gcc::Config::new().get_compiler();

    // Ensure the presence of liblua.a
    if !fs::metadata(build_dir.join(&format!("{}/src/liblua.a", lua_dir))).is_ok() {
        try!(fs::create_dir_all(&build_dir));

        // Download lua if it hasn't been already
        if !fs::metadata(build_dir.join(&lua_tarball)).is_ok() {
            match env::var("LUA_LOCAL_SOURCE") {
                Ok(lua_source_path) => {
                    try!(Command::new("cp")
                         .arg(&PathBuf::from(lua_source_path).join(&lua_tarball))
                         .arg(".")
                         .current_dir(&build_dir)
                         .execute());
                }
                Err(_) => {
                    try!(fetch_in_dir(&format!(
                        "http://www.lua.org/ftp/{}", lua_tarball), &build_dir));
                }
            }
            try!(Command::new("tar")
                .arg("xzf")
                .arg(&lua_tarball)
                .current_dir(&build_dir)
                .execute());
        }
        // Compile lua
        try!(build_lua(&tooling, &build_dir.join(&lua_dir)));
    }

    // Ensure the presence of glue.rs
    if !fs::metadata(build_dir.join("glue.rs")).is_ok() {
        // Compile glue.c
        let glue = build_dir.join("glue");
        try!(Command::new("gcc")
            .arg("-I").arg(build_dir.join(&format!("{}/src", lua_dir)))
            .arg("src/glue/glue.c")
            .arg("-o").arg(&glue)
            .execute());
        try!(Command::new(glue)
            .arg(build_dir.join("glue.rs"))
            .execute());
    }

    // Output build information
    println!("cargo:rustc-link-lib=static=lua");
    println!("cargo:rustc-link-search=native={}/{}/src", build_dir.to_str().unwrap(), lua_dir);

    Ok(())
}

fn main() {
    match prebuild() {
        Err(e) => panic!("Error: {}", e),
        Ok(()) => (),
    }
}
