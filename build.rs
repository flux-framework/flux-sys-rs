use std::env;
use std::path::{Path, PathBuf};

/// Probes for the clang flags needed for the specified Flux component using pkg-config.
/// If pkg-config cannot find the component, the check falls back to using the FLUX_PATH environment variable.
/// Returns a 2-tuple where the first element is a boolean indicating whether the fallback was used (true) or not (false) and
/// the second element is a Vec<String> of the flags that need to be added to Clang.
///
/// Note: If the first element of the return tuple is true, prints related to library linking do not need to be manually
///       injected. The pkg_config crate will do it for us.
fn probe_component_clang_flags(
    component_name: &str,
    min_component_version: &str,
) -> (bool, Vec<String>) {
    match pkg_config::Config::new()
        .atleast_version(min_component_version)
        .probe(component_name)
    {
        Ok(lib) => {
            // Get the include paths for the component
            // Note: we don't need to do any explicit library searching/linking if we hit this
            //       branch because pkg-config does it for us.
            let args: Vec<String> = lib
                .include_paths
                .iter()
                .map(|p| format!("-I{}", p.display()))
                .collect();
            (false, args)
        }
        Err(pkg_err) => {
            // Produce a Cargo warning saying we fell back to FLUX_PATH
            eprintln!(
                "cargo:warning=pkg-config failed for {} ({}), falling back to FLUX_PATH",
                component_name, pkg_err,
            );
            // Check the FLUX_PATH environment variable or panic
            let flux_path = env::var("FLUX_PATH").unwrap_or_else(|_| panic!("{} not found via pkg-config and FLUX_PATH is not set. Set PKG_CONFIG_PATH or FLUX_PATH to your Flux installation", component_name));
            // Tell Clang/LLVM where to search for the component's libraries
            println!("cargo:rustc-link-search=native={}/lib", flux_path);
            let args = vec![format!("-I{}/include", flux_path)];
            (true, args)
        }
    }
}

fn create_base_builder(component_name: &str) -> bindgen::Builder {
    // Create a bindgen builder based on wrapper.h.
    let mut builder = bindgen::Builder::default()
        .header("wrapper.h")
        // Derive useful traits on generated structs
        .derive_debug(true)
        .derive_default(true)
        // Re-run build.rs if wrapper.h changes
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()));

    // Allow the user to pass extra clang args to bindgen using the,
    // BINDGEN_EXTRA_CLANG_ARGS_<component_name> environment variable.
    let env_var_name = format!("BINDGEN_EXTRA_CLANG_ARGS_{}", component_name.to_uppercase());
    if let Ok(extra_args) = env::var(env_var_name) {
        for arg in extra_args.split_whitespace() {
            builder = builder.clang_arg(arg);
        }
    }

    builder
}

macro_rules! generate_bindings {
    ($builder:expr, $component_name:literal) => {
        $builder.generate().unwrap_or_else(|err| panic!(
            "Unable to generate bindings. Ensure {} is installed and discoverable via pkg-config or FLUX_PATH. Error: {}",
            $component_name, err,
        ))
    };
}

macro_rules! write_bindings {
    ($bindings:expr, $binding_file_name:literal, $out_dir:expr) => {{
        $bindings
            .write_to_file($out_dir.join($binding_file_name))
            .unwrap_or_else(|err| {
                panic!(
                    "Cannot write bindings to {}. Error: {}",
                    $binding_file_name, err
                )
            });
    }};
}

fn configure_core(out_dir: &Path) {
    let (is_fallback, include_args) = probe_component_clang_flags("flux-core", "0.49.0");
    if is_fallback {
        println!("cargo:rustc-link-lib=flux-core");
    }
    let bindings = generate_bindings!(
        create_base_builder("flux-core")
            .clang_args(&include_args)
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
            .constified_enum_module("kvs_op")
            // Define a macro to make sure Bindgen can see contents in wrapper.h
            .clang_arg("-DFLUX_SYS_FEATURE_CORE"),
        "flux-core"
    );
    write_bindings!(bindings, "core.rs", out_dir);
}

fn configure_idset(out_dir: &Path) {
    let (is_fallback, include_args) = probe_component_clang_flags("flux-idset", "0.49.0");
    if is_fallback {
        println!("cargo:rustc-link-lib=flux-idset");
    }
    let bindings = generate_bindings!(
        create_base_builder("flux-idset")
            .clang_args(&include_args)
            .allowlist_function("idset_.*")
            .allowlist_type("idset_.*")
            .allowlist_var("IDSET_.*")
            .allowlist_var("idset_.*")
            .constified_enum_module("idset_flags")
            // Define a macro to make sure Bindgen can see contents in wrapper.h
            .clang_arg("-DFLUX_SYS_FEATURE_IDSET"),
        "flux-idset"
    );
    write_bindings!(bindings, "idset.rs", out_dir);
}

fn configure_hostlist(out_dir: &Path) {
    let (is_fallback, include_args) = probe_component_clang_flags("flux-hostlist", "0.49.0");
    if is_fallback {
        println!("cargo:rustc-link-lib=flux-hostlist");
    }
    let bindings = generate_bindings!(
        create_base_builder("flux-hostlist")
            .clang_args(&include_args)
            .allowlist_function("hostlist_.*")
            .allowlist_type("hostlist_.*")
            // Define a macro to make sure Bindgen can see contents in wrapper.h
            .clang_arg("-DFLUX_SYS_FEATURE_HOSTLIST"),
        "flux-hostlist"
    );
    write_bindings!(bindings, "hostlist.rs", out_dir);
}

fn main() {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    // Generate bindings for each feature
    // Add blocks as needed for new features
    if cfg!(feature = "core") {
        configure_core(&out_dir);
    }
    if cfg!(feature = "idset") {
        configure_idset(&out_dir);
    }
    if cfg!(feature = "hostlist") {
        configure_hostlist(&out_dir);
    }

    // Re-run if relevant environment variables change
    println!("cargo:rerun-if-env-changed=FLUX_PATH");
    println!("cargo:rerun-if-env-changed=PKG_CONFIG_PATH");
    println!("cargo:rerun-if-env-changed=BINDGEN_EXTRA_CLANG_ARGS");
    println!("cargo:rerun-if-changed=wrapper.h");
}
