use std::env;
use std::path::PathBuf;

fn main() {
    // Generate bindings to KLLVM's C API
    let bindings = bindgen::Builder::default()
        .header("c/kllvm-c.h")
        .allowlist_file("c/kllvm-c.h")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()
        .expect("Unable to generate kllvm-c bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("kllvm-c.rs"))
        .expect("Couldn't write kllvm-c bindings!");
}
