#![feature(exit_status_error)]
use std::{env, error, path::Path, process::Command};

fn main() -> Result<(), Box<dyn error::Error>> {
    let out_dir = env::var("OUT_DIR").unwrap();

    Command::new("cc")
        .args(["-Wall", "-Werror"])
        .args(["-c", "csrc/xcbshow.c"])
        .args(["-fPIC"])
        .args(["-o", &format!("{}/xcbshow.o", out_dir)])
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
        .clang_arg("-I/usr/lib64/gcc/x86_64-suse-linux/12/include/")
        .allowlist_type("window_t")
        .allowlist_type("image_t")
        .allowlist_type("event_t")
        .allowlist_function("create_image")
        .allowlist_function("show_image")
        .allowlist_function("resize_image")
        .allowlist_function("destroy_image")
        .allowlist_function("create_window")
        .allowlist_function("destroy_window")
        .allowlist_function("wait_for_event")
        .allowlist_function("destroy_event")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .generate()?
        .write_to_file(Path::new(&out_dir).join("bindings.rs"))?;

    Ok(())
}
