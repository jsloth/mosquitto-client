/// this file is mainly inspired by the great c library integration from https://github.com/eclipse/paho.mqtt.rust

fn main() {
    bundled::main();
}

const MOSQUITTO_VERSION: &str = "2.0.4";

#[cfg(feature = "build_bindgen")]
mod bindings {
    use std::{env, fs};
    use std::path::{Path, PathBuf};
    use MOSQUITTO_VERSION;

    pub fn place_bindings(inc_dir: &Path) {
        let inc_search = format!("-I{}", inc_dir.display());

        // The bindgen::Builder is the main entry point
        // to bindgen, and lets you build up options for
        // the resulting bindings.
        let bindings = bindgen::Builder::default()
            // Older clang versions (~v3.6) improperly mangle the functions.
            // We shouldn't require mangling for straight C library. I think.
            .trust_clang_mangling(false)
            // The input header we would like to generate
            // bindings for.
            .header("wrapper.h").clang_arg(inc_search)
            // Finish the builder and generate the bindings.
            .generate()
            // Unwrap the Result and panic on failure.
            .expect("Unable to generate bindings");

        // Write the bindings to the $OUT_DIR/bindings.rs file.
        let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
        let out_path = out_dir.join("bindings.rs");

        bindings
            .write_to_file(out_path.clone())
            .expect("Couldn't write bindings!");

        // Save a copy of the bindings file into the bindings/ dir
        // with version and target name, if it doesn't already exist

        let target = env::var("TARGET").unwrap();
        println!("debug:Target: {}", target);

        let bindings = format!("bindings/bindings_mosquitto_{}-{}.rs",
                               MOSQUITTO_VERSION, target);

        if !Path::new(&bindings).exists() {
            if let Err(err) = fs::copy(out_path, &bindings) {
                println!("debug:Error copying new binding file: {}", err);
            } else {
                println!("debug:Created new bindings file {}", bindings)
            }
        }
    }
}

#[cfg(feature = "bundled")]
mod bundled {
    use std::process::Command;
    use super::*;
    use std::path::Path;
    use std::process;

    extern crate cmake;

    pub fn main() {
        println!("Running the bundled build");
        if let Err(e) = Command::new("git").args(&["submodule", "update", "--init"]).status() {
            println!("failed to update the git submodule: {:?}", e);
        }

        let mut cmk_cfg = cmake::Config::new("mosquitto");
        let cmk = cmk_cfg.build();

        let lib_path = if cmk.join("lib").exists() {
            "lib"
        } else {
            panic!("Unknown library directory.")
        };

        let lib_dir = cmk.join(lib_path);

        let library_name = "mosquitto";
        let link_file = format!("lib{}.so.1", library_name);

        let lib = lib_dir.join(Path::new(&link_file));
        println!("debug:Using mosquitto C library at: {}", lib.display());

        if !lib.exists() {
            println!("Error building mosquitto C library: '{}'", lib.display());
            process::exit(103);
        }

        // Get bundled bindings or regenerate
        let inc_dir = cmk.join("include");
        bindings::place_bindings(&inc_dir);

        // we add the folder where all the libraries are built to the path search
        println!("cargo:rustc-link-search=native={}", lib_dir.display());
        println!("cargo:rustc-link-lib={}", "mosquitto");
    }
}
