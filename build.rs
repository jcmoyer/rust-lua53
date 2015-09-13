use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;

#[cfg(target_os="windows")]
fn build_lua() -> io::Result<()> {
    run_command_in_dir(&["make", "mingw"], Some(&build_dir().join("lua-5.3.0")))
}

/// The comand to build lua, specialized for different OSes.
#[cfg(target_os="macos")]
fn build_lua() -> io::Result<()> {
    run_command_in_dir(&["make", "macosx"], Some(&build_dir().join("lua-5.3.0")))
}

#[cfg(target_os="linux")]
fn build_lua() -> io::Result<()> {
    let dir = build_dir().join("lua-5.3.0");
    try!(run_command_in_dir(&["sed", "-i", "s/^MYCFLAGS=.*/MYCFLAGS=-fPIC/g", "src/Makefile"],
                            Some(&dir)));
    run_command_in_dir(&["make", "linux"], Some(&dir))
}

#[cfg(any(target_os = "freebsd", target_os = "dragonfly"))]
fn build_lua() -> io::Result<()> {
    let dir = build_dir().join("lua-5.3.0");
    try!(run_command_in_dir(&["sed", "-e", "s/^MYCFLAGS=.*/MYCFLAGS=-fPIC/g", "-i", "bak", "src/Makefile"],
                            Some(&dir)));
    if cfg!(target_os = "freebsd") {
        run_command_in_dir(&["make", "freebsd"], Some(&dir))
    } else {
        run_command_in_dir(&["make", "bsd"], Some(&dir))
    }
}

/// The command to fetch a URL (e.g. with wget) specialized for different
/// OSes.
#[cfg(not(any(target_os = "freebsd", target_os = "dragonfly", target_os = "macos")))]
fn fetch_in_dir(url: &str, cwd: Option<&Path>) -> io::Result<()> {
    run_command_in_dir(&["wget", url], cwd)
}

#[cfg(any(target_os = "freebsd", target_os = "dragonfly"))]
fn fetch_in_dir(url: &str, cwd: Option<&Path>) -> io::Result<()> {
    run_command_in_dir(&["fetch", url], cwd)
}

#[cfg(target_os = "macos")]
fn fetch_in_dir(url: &str, cwd: Option<&Path>) -> io::Result<()> {
    run_command_in_dir(&["curl", "-O", url], cwd)
}

/// Runs the command 'all_args[0]' with the arguments 'all_args[1..]' in the
/// current directory.
fn run_command(all_args: &[&str]) -> io::Result<()> {
    run_command_in_dir(all_args, None)
}

/// Runs the command 'all_args[0]' with the arguments 'all_args[1..]' in the
/// directory 'cwd' or the current directory.
fn run_command_in_dir(all_args: &[&str], cwd: Option<&Path>) -> io::Result<()> {
    let command_name = all_args[0];
    let args = &all_args[1..];
    let mut command = Command::new(command_name);
    command.args(args);
    if let Some(cwd) = cwd {
        command.current_dir(cwd);
    }
    let status = try!(command.status());
    if !status.success() {
        return Err(io::Error::new(io::ErrorKind::Other, format!("The command\n\
        \t{}\n\
        did not run successfully.", all_args.connect(" "))));
    }
    Ok(())
}

/// The cargo output directory for builds from this.
fn build_dir() -> PathBuf {
    PathBuf::from(env::var("OUT_DIR").unwrap())
}

/// If a static Lua is not yet available from a prior run of this script, this
/// will download Lua and build it. The cargo configuration text to link
/// statically against lua.a is then printed to stdout.
fn prebuild() ->io::Result<()> {
    if !fs::metadata(
        &build_dir().join("lua-5.3.0").join("src").join("liblua.a")
    ).is_ok() {
        try!(fs::create_dir_all(&build_dir()));


        // Compile Lua.
        try!(fetch_in_dir("http://www.lua.org/ftp/lua-5.3.0.tar.gz",
                          Some(&build_dir())));
        try!(run_command_in_dir(&["tar", "xzvf", "lua-5.3.0.tar.gz"],
                                Some(&build_dir())));

        try!(build_lua());

        // Compile and run glue.c.
        let glue = build_dir().join("glue").to_string_lossy().into_owned();
        try!(run_command(&["gcc",
                         "-I", &build_dir().join("lua-5.3.0").join("src").to_string_lossy(),
                         &PathBuf::from("src").join("glue").join("glue.c").to_string_lossy(),
                         "-o", &glue]));
        try!(run_command(&[&glue, "src/ffi/glue.rs"]));
    }

    let mut build_dir_absolute = try!(env::current_dir());
    build_dir_absolute.push(&build_dir());
    build_dir_absolute.push("lua-5.3.0");
    build_dir_absolute.push("src");

    println!("cargo:rustc-link-lib=static=lua");
    println!("cargo:rustc-link-search=native={}", build_dir_absolute.to_string_lossy());

    Ok(())
}

fn main() {
    match prebuild() {
        Err(e) => panic!("Error: {}", e),
        Ok(()) => (),
    }
}
