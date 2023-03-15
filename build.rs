fn main() {
    println!("cargo:rustc-link-lib=dylib=vg_lite");
    println!("cargo:rustc-link-search=native=./");
}
