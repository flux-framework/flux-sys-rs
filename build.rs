extern crate bindgen;

use which::which;
use std::env;
use std::path::PathBuf;

fn main() {
    // Tell cargo to tell rustc to link the system bzip2
    // shared library.
    println!("cargo:rustc-link-lib=flux-core");

    if let Ok(mut clang_path) = which("clang") {
        println!("{:?}", clang_path);
        if let Err(_) = env::var("CLANG_PATH") {
            env::set_var("CLANG_PATH", &clang_path);
        }
        if let Err(_) = env::var("LIBCLANG_PATH") {
            // TODO: deal with 32 bit?
            clang_path.pop(); // remove clang
            clang_path.pop(); // remove bin
            clang_path.push("lib64");
            env::set_var("LIBCLANG_PATH", clang_path);
        }
    }

    // The bindgen::Builder is the main entry point
    // to bindgen, and lets you build up options for
    // the resulting bindings.
    let bindings = bindgen::Builder::default()
        // The input header we would like to generate
        // bindings for.
        .header("wrapper.h")
        // Only take flux functions, etc.
        .whitelist_function("flux_.*")
        .whitelist_var("FLUX_.*")
        .whitelist_var("flux_.*")
        .whitelist_type("flux_.*")
        .whitelist_type("kvs_.*")
        .constified_enum_module("flux_flag")
        .constified_enum_module("kvs_op")
        // .rustified_enum("kvs_op") //can't, enums need names and functions need to take the enum
        // .bitfield_enum("FLUX_.*")
        // .rustified_enum("FLUX_.*") //can't, enums need names and functions need to take the enum
        // type...
        // Finish the builder and generate the bindings.
        .generate()
        // Unwrap the Result and panic on failure.
        .expect("Unable to generate bindings");

    // Write the bindings to the $OUT_DIR/bindings.rs file.
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
