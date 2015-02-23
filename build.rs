#![feature(process, path, env)]
use std::process::Command;
use std::env::current_dir;

/*
 NOTES:

-- LLVM-CONFIG INVOCATION --
llvm-config --libs --cflags --ldflags core analysis executionengine mcjit interpreter native
*/

fn main() {
    // Build the ducky runtime
    assert!(Command::new("clang")
        .args(&["rt/rt.c", "-c", "-emit-llvm", "-O3", "-o", "rt/rt.bc"])
        .status().unwrap().success());

    // Get the configuration for binding to llvm
    let config = Command::new("llvm-config")
        .args(&["--libs", "--cflags", "--ldflags",
                "core", "analysis", "executionengine", "mcjit", "interpreter", "native"])
        .output().unwrap_or_else(|e| {
            panic!("Failed to execute process: {}", e);
        });
    assert_eq!(config.status.code(), Some(0));

    // Split the options apart
    let config_str = String::from_utf8(config.stdout).unwrap();
    let config: Vec<_> = config_str.split(|c: char| c.is_whitespace()).collect();

    // Build & call bindgen to create the bindings
    assert!(Command::new("cargo")
        .args(&["run", "--"])
        .current_dir("vendor/rust-bindgen")
        // Configuration Options
        .args(&config)
        .arg("-builtins")
        // .args(&["-match Core.h"])
        // Output File
        .arg("-o").arg(&current_dir().unwrap().join("src/gen/llvm/ffi.rs"))
        // Input File
        .arg(&current_dir().unwrap().join("src/gen/llvm/ffi-header.h"))
        .status().unwrap().success());

    // Link to stdc++ (and curses for some reason)
    println!("cargo:rustc-flags=-l stdc++ -l curses");
}
