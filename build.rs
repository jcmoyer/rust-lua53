use std::fs;
use std::io;
use std::path::Path;
use std::process::Command;

/// The command to build lua, with switches for different OSes.
fn build_lua(dir: &Path) -> io::Result<()> {
    let platform = if cfg!(target_os = "windows") {
        "mingw"
    } else if cfg!(target_os = "macos") {
        "macosx"
    } else if cfg!(target_os = "linux") {
        "linux"
    } else if cfg!(target_os = "freebsd") {
        "freebsd"
    } else if cfg!(target_os = "dragonfly") {
        "bsd"
    } else {
        panic!("Unsupported target OS")
    };

    if cfg!(any(target_os = "linux", target_os = "freebsd", target_os = "bsd")) {
        run_command(&["make", platform, "MYCFLAGS=-fPIC"], Some(dir))
    } else {
        run_command(&["make", platform], Some(dir))
    }
}

/// The command to fetch a URL (e.g. with wget) specialized for different
/// OSes.
#[cfg(not(any(target_os = "freebsd", target_os = "dragonfly", target_os = "macos")))]
fn fetch_in_dir(url: &str, cwd: Option<&Path>) -> io::Result<()> {
    run_command(&["wget", url], cwd)
}

#[cfg(any(target_os = "freebsd", target_os = "dragonfly"))]
fn fetch_in_dir(url: &str, cwd: Option<&Path>) -> io::Result<()> {
    run_command(&["fetch", url], cwd)
}

#[cfg(target_os = "macos")]
fn fetch_in_dir(url: &str, cwd: Option<&Path>) -> io::Result<()> {
    run_command(&["curl", "-O", url], cwd)
}

/// Runs the command 'all_args[0]' with the arguments 'all_args[1..]' in the
/// directory 'cwd' or the current directory.
fn run_command(all_args: &[&str], cwd: Option<&Path>) -> io::Result<()> {
    let mut command = Command::new(all_args[0]);
    command.args(&all_args[1..]);
    if let Some(cwd) = cwd {
        command.current_dir(cwd);
    }
    let status = try!(command.status());
    if !status.success() {
        return Err(io::Error::new(io::ErrorKind::Other, format!("The command\n\
        \t{}\n\
        did not run successfully.", all_args.join(" "))));
    }
    Ok(())
}

/// If a static Lua is not yet available from a prior run of this script, this
/// will download Lua and build it. The cargo configuration text to link
/// statically against lua.a is then printed to stdout.
fn prebuild() -> io::Result<()> {
    let build_dir = Path::new(env!("OUT_DIR"));

    // Ensure the presence of liblua.a
    if !fs::metadata(concat!(env!("OUT_DIR"), "/lua-5.3.0/src/liblua.a")).is_ok() {
        try!(fs::create_dir_all(build_dir));

        // Download lua if it hasn't been already
        if !fs::metadata(concat!(env!("OUT_DIR"), "/lua-5.3.0.tar.gz")).is_ok() {
            try!(fetch_in_dir("http://www.lua.org/ftp/lua-5.3.0.tar.gz", Some(build_dir)));
            try!(run_command(&["tar", "xzf", "lua-5.3.0.tar.gz"], Some(build_dir)));
        }
        // Compile lua
        try!(build_lua(Path::new(concat!(env!("OUT_DIR"), "/lua-5.3.0"))));
    }

    // Ensure the presence of glue.rs
    if !fs::metadata(concat!(env!("OUT_DIR"), "/glue.rs")).is_ok() {
        // Compile glue.c
        let glue = concat!(env!("OUT_DIR"), "/glue");
        try!(run_command(&["gcc",
                         "-I", concat!(env!("OUT_DIR"), "/lua-5.3.0/src"),
                         "src/glue/glue.c",
                         "-o", &glue], None));
        // Run glue to generate glue.rs
        try!(run_command(&[&glue, concat!(env!("OUT_DIR"), "/glue.rs")], None));
    }

    // Output build information
    println!("cargo:rustc-link-lib=static=lua");
    println!(concat!("cargo:rustc-link-search=native=", env!("OUT_DIR"), "/lua-5.3.0/src"));

    Ok(())
}

fn main() {
    match prebuild() {
        Err(e) => panic!("Error: {}", e),
        Ok(()) => (),
    }
}
