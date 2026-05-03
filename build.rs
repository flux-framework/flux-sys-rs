use std::env;
use std::path::PathBuf;

fn main() {
    // Find flux-core.
    // By default, this uses pkg-config. If that lookup succeeds,
    // we simply build the include compiler flags and store them.
    // If the lookup fails, we fallback to a manual lookup using the
    // FLUX_PATH environment variable.
    let include_args: Vec<String> = match pkg_config::Config::new()
        .atleast_version("0.49.0")
        .probe("flux-core")
    {
        Ok(lib) => {
            // pkg-config found flux-core. It has already emitted
            // cargo:rustc-link-lib and cargo:rustc-link-search
            // directives for us.
            let args: Vec<String> = lib
                .include_paths
                .iter()
                .map(|p| format!("-I{}", p.display()))
                .collect();
            args
        }
        Err(pkg_err) => {
            // Fall back to FLUX_PATH, matching the old behavior
            eprintln!(
                "cargo:warning=pkg-config failed ({}), falling back to FLUX_PATH",
                pkg_err
            );

            let flux_path = env::var("FLUX_PATH").unwrap_or_else(|_| {
                panic!(
                    "flux-core not found via pkg-config and FLUX_PATH is not set. \
                     Set PKG_CONFIG_PATH or FLUX_PATH to your Flux installation."
                )
            });

            println!("cargo:rustc-link-lib=flux-core");
            println!("cargo:rustc-link-search=native={}/lib", flux_path);

            let args = vec![format!("-I{}/include", flux_path)];
            args
        }
    };

    // Create a bindgen builder based on wrapper.h.
    // By default, we only set the -I flags from above as clang args.
    let mut builder = bindgen::Builder::default()
        .header("wrapper.h")
        .clang_args(&include_args);

    // Allow the user to pass extra clang args to bindgen using the,
    // BINDGEN_EXTRA_CLANG_ARGS environment variable.
    if let Ok(extra_args) = env::var("BINDGEN_EXTRA_CLANG_ARGS") {
        for arg in extra_args.split_whitespace() {
            builder = builder.clang_arg(arg);
        }
    }

    // Configure bindgen filters
    let bindings = builder
        // Functions
        .allowlist_function("flux_.*")
        // Types
        .allowlist_type("flux_.*")
        .allowlist_type("kvs_.*")
        // Constants and variables
        .allowlist_var("FLUX_.*")
        .allowlist_var("flux_.*")
        // Enum handling: constified_enum_module creates a Rust module
        // with associated constants, which is safer than rustified_enum
        // for C enums that are used as bitflags or passed as int arguments
        .constified_enum_module("flux_flag")
        .constified_enum_module("kvs_op")
        // Derive useful traits on generated structs
        .derive_debug(true)
        .derive_default(true)
        // Re-run build.rs if wrapper.h changes
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        // Generate bindings
        .generate()
        // Error out if anything goes wrong
        .expect(
            "Unable to generate bindings. Ensure flux-core is installed \
             and discoverable via pkg-config or FLUX_PATH.",
        );

    // Write bindings to $OUT_DIR
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("core.rs"))
        .expect("Couldn't write bindings");

    // Re-run if relevant environment variables change
    println!("cargo:rerun-if-env-changed=FLUX_PATH");
    println!("cargo:rerun-if-env-changed=PKG_CONFIG_PATH");
    println!("cargo:rerun-if-env-changed=BINDGEN_EXTRA_CLANG_ARGS");
    println!("cargo:rerun-if-changed=wrapper.h");
}
