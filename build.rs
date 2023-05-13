fn main() {
    let path = std::path::PathBuf::from("lib/GamepadMotionHelpers");
    let b = autocxx_build::Builder::new("src/main.rs", &[&path])
        .extra_clang_args(&["-std=c++17"])
        .build();
    b.unwrap()
        .flag_if_supported("-Wno-comment")
        .flag_if_supported("-Wno-unused-parameter")
        .compile("aimu");
    println!("cargo:rerun-if-changed=src/main.rs");
    println!("cargo:rerun-if-changed=lib/GamepadMotionHelpers/GamepadMotion.hpp");
}
