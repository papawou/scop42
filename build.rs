use std::env;
use std::path::PathBuf;

const SRC_FOLDER: &str = "./bin/nvidia_aftermath";

fn main() {
    // Link against the Nsight Aftermath SDK
    println!("cargo:rustc-link-lib=dylib=GFSDK_Aftermath_Lib.x64");
    println!("cargo:rustc-link-search={SRC_FOLDER}/lib/x64");

    // Generate bindings
    let bindings = bindgen::Builder::default()
        .header(format!("{}/include/GFSDK_Aftermath.h", SRC_FOLDER))
        .generate_comments(false)
        .default_enum_style(bindgen::EnumVariation::Rust {
            non_exhaustive: false,
        })
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("aftermath_bindings.rs"))
        .expect("Couldn't write bindings!");
}
