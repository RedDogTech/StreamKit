fn main() {
    #[cfg(target_os = "macos")]
    {
        println!("cargo:rustc-link-search=/opt/homebrew/Cellar/srt/1.5.2/lib/");
        println!("cargo:rustc-link-lib=c++");
    }

    #[cfg(target_os = "linux")]
    {
        println!("cargo:rustc-link-search={}/vendor/linux/lib", env!("CARGO_MANIFEST_DIR"));
        println!("cargo:rustc-link-lib=stdc++");
        println!("cargo:rustc-link-lib=crypto");
    }
}