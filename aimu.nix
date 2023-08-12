{ lib
, pkgs
, rustPlatform
}:
rustPlatform.buildRustPackage {
  pname = "aimu";
  version = "unstable";

  src = lib.cleanSource ./.;

  cargoLock = {
    lockFile = ./Cargo.lock;
    outputHashes = {
      "bmi270-0.1.0" = "sha256-LZBYu4kXHzCmIQvS3iQCUv3MOxBgNUXtCCpym4tGRXQ=";
      "gamepad_motion-0.1.2" = "sha256-lCOqlQpbi79Lj+9aVFhG7Hi6SgktVdxYR7CH1pppAzQ=";
    };
  };

  LIBCLANG_PATH = "${pkgs.llvmPackages.libclang.lib}/lib";

  preBuild = with pkgs; ''
    # From: https://github.com/NixOS/nixpkgs/blob/1fab95f5190d087e66a3502481e34e15d62090aa/pkgs/applications/networking/browsers/firefox/common.nix#L247-L253
    # Set C flags for Rust's bindgen program. Unlike ordinary C
    # compilation, bindgen does not invoke $CC directly. Instead it
    # uses LLVM's libclang. To make sure all necessary flags are
    # included we need to look in a few places.
    export BINDGEN_EXTRA_CLANG_ARGS="$(< ${stdenv.cc}/nix-support/libc-crt1-cflags) \
      $(< ${stdenv.cc}/nix-support/libc-cflags) \
      $(< ${stdenv.cc}/nix-support/cc-cflags) \
      $(< ${stdenv.cc}/nix-support/libcxx-cxxflags) \
      ${lib.optionalString stdenv.cc.isClang "-idirafter ${stdenv.cc.cc}/lib/clang/${lib.getVersion stdenv.cc.cc}/include"} \
      ${lib.optionalString stdenv.cc.isGNU "-isystem ${stdenv.cc.cc}/include/c++/${lib.getVersion stdenv.cc.cc} -isystem ${stdenv.cc.cc}/include/c++/${lib.getVersion stdenv.cc.cc}/${stdenv.hostPlatform.config} -idirafter ${stdenv.cc.cc}/lib/gcc/${stdenv.hostPlatform.config}/${lib.getVersion stdenv.cc.cc}/include"} \
    "
  '';

  nativeBuildInputs = with pkgs; [ llvmPackages.clang pkg-config ];

  buildFeatures = [ "dynamic" "cli" ];

  meta = {
    description = "Userspace IMU-assisted aiming for Linux.";
    homepage = "https://gitlab.com/awahab/aimu";
    license = lib.licenses.mit;
    maintainers = [ "shymega" ];
  };
}
