#![feature(exit_status_error)]
use std::{env, error, path::Path, process::Command};

fn main() -> Result<(), Box<dyn error::Error>> {
    let out_dir = env::var("OUT_DIR")?;

    let target = format!("{}/xcbshow.o", out_dir);
    let mut ccargs = vec![
        "-Wall",
        "-Werror",
        "-c",
        "csrc/xcbshow.c",
        "-fPIC",
        "-o",
        &target,
    ];
    if env::var("PROFILE")? == "debug" {
        ccargs.push("-DDEBUG=1");
        ccargs.push("-g")
    } else {
        ccargs.push("-O3");
    }
    Command::new("cc")
        .args(ccargs)
        .spawn()?
        .wait_with_output()?
        .status
        .exit_ok()?;

    Command::new("ar")
        .args(&["crus", "libxcbshow.a", "xcbshow.o"])
        .current_dir(&Path::new(&out_dir))
        .status()?;

    println!("cargo:rustc-link-search=native={}", out_dir);
    println!("cargo:rustc-link-lib=static=xcbshow");
    println!("cargo:rustc-link-lib=xcb-image");
    println!("cargo:rustc-link-lib=xcb");
    println!("cargo:rerun-if-changed=xcbshow.h");

    bindgen::Builder::default()
        .header("csrc/xcbshow.h")
        .clang_arg(format!(
            "-I{}",
            env::var("GCC_INCLUDE").map_err(|e| format!("GCC_INCLUDE {}", e))?
        ))
        .allowlist_file("csrc/xcbshow.h")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .generate()?
        .write_to_file(Path::new(&out_dir).join("bindings.rs"))?;

    Ok(())
}
