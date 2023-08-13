{ lib
, naersk
, stdenv
, clangStdenv
, hostPlatform
, targetPlatform
, pkg-config
, libiconv
, rustfmt
, cargo
, rustc
, llvmPackages
, libcxx
, glibc
}:

let
  cargoToml = (builtins.fromTOML (builtins.readFile ./Cargo.toml));
in

naersk.lib."${targetPlatform.system}".buildPackage rec {
  src = ./.;

  buildInputs = [
    rustfmt
    pkg-config
    cargo
    rustc
  ];
  checkInputs = [ cargo rustc ];

  doCheck = true;
  copyTarget = true;
  copyLibs = true;
  gitSubmodules = true;

  RUST_BACKTRACE = "full";
  LIBCLANG_PATH = lib.makeLibraryPath [ llvmPackages.libclang.lib ];
  BINDGEN_EXTRA_CLANG_ARGS = [
    ''-I"${glibc.dev}/include"''
    ''-I"${libcxx.dev}/include/c++/v1"''
    ''-I"${llvmPackages.libclang.lib}/lib/clang/${llvmPackages.libclang.version}/include"''
  ];

  name = cargoToml.package.name;
  version = cargoToml.package.version;

  meta = with lib; {
    #    description = cargoToml.package.description;
    #    homepage = cargoToml.package.homepage;
    #    license = with licenses; [ mit ];
    maintainers = with maintainers; [ ];
  };
}
