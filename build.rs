use std::process::Command;

fn main() {
  Command::new("sh").arg("prebuild.sh").status().unwrap();
}

