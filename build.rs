use std::old_io::Command;

fn main() {
  Command::new("sh").arg("prebuild.sh").status().unwrap();
}

