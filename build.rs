extern crate pkg_config;

use std::process::Command;
use std::fs;
use std::io;

fn run_command(all_args: &[&str]) -> io::Result<()> {
    let command = all_args[0];
    let args = &all_args[1..];
    let status = try!(Command::new(command).args(args).status());
    if !status.success() {
        return Err(io::Error::new(io::ErrorKind::Other, format!("The command\n\
        \t{}\n\
        did not run successfully.", all_args.connect(" "))));
    }
    Ok(())
}

fn cleanup() -> io::Result<()> {
    try!(fs::remove_dir_all("build"));
    Ok(())
}

fn prebuild(include_dir: &str) ->io::Result<()> {
    // Ignore errors on cleanup.
    let _ = cleanup();

    try!(fs::create_dir("build"));

    // Compile and run build/checkver
    try!(run_command(&["gcc",
         "-I", include_dir,
         "src/glue/checkver.c", "-o", "build/checkver"]));
    try!(run_command(&["build/checkver"]).map_err(|_| {
        io::Error::new(io::ErrorKind::Other, format!(
          "Bad Lua version. Ensure that LUA_INCLUDE points to a directory containing \
          Lua 5.3 header files."))
    }));

    // Compile and run glue.c.
    try!(run_command(&["gcc",
         "-I", include_dir,
         "src/glue/glue.c", "-o", "build/glue"]));
    try!(run_command(&["build/glue", "src/ffi/glue.rs"]));

    Ok(())
}

fn main() {
    let lua_config = pkg_config::find_library("lua-5.3").unwrap();

    let result = prebuild(&lua_config.include_paths[0].to_string_lossy())
        .and(cleanup());

    match result {
        Err(e) => panic!("Error: {}", e),
        Ok(()) => (),
    }
}
