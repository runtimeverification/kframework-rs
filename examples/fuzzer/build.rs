use std::env;

fn main() {
    // Link interpreter.so
    println!("cargo:rustc-link-arg=-l:interpreter.so");

    // Add the location of interpreter.so to your LD_LIBRARY_PATH variable
    // You will also need this variable set when you run the program
    if let Some(paths) = env::var_os("LD_LIBRARY_PATH") {
            for lib_path in env::split_paths(&paths) {
                println!("cargo:rustc-link-search={}", lib_path.display());
            }
    };
}
