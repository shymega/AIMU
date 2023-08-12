{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-24.05";
    flake-utils.url = "github:numtide/flake-utils";
    flake-compat = {
      url = "github:edolstra/flake-compat";
      flake = false;
    };
  };

  outputs =
    { self
    , nixpkgs
    , flake-utils
    , ...
    }:
    flake-utils.lib.eachDefaultSystem
      (system:
      let
        pkgs = nixpkgs.outputs.legacyPackages.${system};
      in
      {
        packages.aimu = pkgs.callPackage ./aimu.nix { };
        packages.default = self.outputs.packages.${system}.aimu;
      }) // {
      overlays.default = final: prev: {
        inherit (self.packages.${final.system}) aimu;
      };
    };
}
