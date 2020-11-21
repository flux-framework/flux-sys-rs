extern crate bindgen;

use std::env;
use std::path::PathBuf;
use std::process::Command;
use which::which;

fn get_path_from_llvm(p: &str, s: &[&str]) -> PathBuf {
    PathBuf::from(
        String::from_utf8(
            Command::new(p)
                .args(s)
                .output()
                .expect("llvm-config --libdir failed")
                .stdout,
        )
        .expect("response from llvm-config is not valid utf8")
        .trim(),
    )
}

fn main() {
    // Tell cargo to tell rustc to link the system bzip2
    // shared library.
    println!("cargo:rustc-link-lib=flux-core");

    if let Ok(llvm_config_path) = which("llvm-config") {
        let lc_str = llvm_config_path
            .to_str()
            .expect("llvm config path is not valid utf8");
        let libdir = get_path_from_llvm(lc_str, &["--libdir"]);
        let mut clang_path = get_path_from_llvm(lc_str, &["--bindir"]);

        clang_path.push("clang");
        println!("LIBCLANG_PATH={:?}", libdir);
        env::set_var("LIBCLANG_PATH", libdir);
        println!("CLANG_PATH={:?}", clang_path);
        env::set_var("CLANG_PATH", clang_path);
    } else if let Ok(mut clang_path) = which("clang") {
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

    let flux_path = match env::var("FLUX_PATH") {
        Ok(p) => p,
        Err(_) => "/usr/local".to_string(),
    };
    let include_arg = "-I".to_string() + &flux_path + "/include";
    println!("cargo:rustc-link-search=native={}/lib", flux_path);

    // The bindgen::Builder is the main entry point
    // to bindgen, and lets you build up options for
    // the resulting bindings.
    let bindings = bindgen::Builder::default()
        .clang_arg(include_arg)
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
        // .bitfield_enum("FLUX_.*")
        // .rustified_enum("FLUX_.*") //can't, enums need names and functions need to take the enum
        // type...
        // Finish the builder and generate the bindings.
        .generate()
        // Unwrap the Result and panic on failure.
        .expect(
            "Unable to generate bindings.  If this is an include directory or clang argument
        issue, use BINDGEN_EXTRA_CLANG_ARGS='args' to pass in necessary include paths and FLUX_PATH
        to set the base path of the flux install",
        );

    // Write the bindings to the $OUT_DIR/bindings.rs file.
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
