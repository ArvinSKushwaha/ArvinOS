fn main() {
    println!("cargo:rustc-link-args=-Tlinker.ld");
    println!("cargo:rustc-link-args=-nostdlib");
}
