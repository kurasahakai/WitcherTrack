fn main() {
    println!("cargo:rustc-link-lib=static=archive");
    println!("cargo:rustc-link-lib=User32");
    println!("cargo:rustc-link-lib=Crypt32");
}
