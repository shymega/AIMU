fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::path::PathBuf::from("lib/GamepadMotionHelpers");
    let mut b = autocxx_build::Builder::new("src/motion.rs", [&path])
        .extra_clang_args(&["-std=c++17"])
        .build()?;
    b.flag_if_supported("-std=c++17")
        .flag_if_supported("-Wno-comment")
        .flag_if_supported("-Wno-unused-parameter")
        .compile("aimu");
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=src/");
    println!("cargo:rerun-if-changed=lib/GamepadMotionHelpers/GamepadMotion.hpp");
    Ok(())
}
