#![feature(io)]
use std::old_io::Command;

fn main() {
    // let out_dir = os::getenv("OUT_DIR").unwrap();
    // Build the runtime!
    Command::new("clang")
        .args(&["rt/rt.c", "-c", "-emit-llvm", "-O3", "-o", "rt/rt.bc"])
        .status().unwrap();
}
