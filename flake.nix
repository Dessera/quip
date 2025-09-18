{
  description = "Simple chat protocol.";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

    flake-parts.url = "github:hercules-ci/flake-parts";
    rust-flake.url = "github:juspay/rust-flake";
  };

  outputs =
    {
      flake-parts,
      rust-flake,
      ...
    }@inputs:
    flake-parts.lib.mkFlake { inherit inputs; } {
      systems = [ "x86_64-linux" ];
      imports = [
        rust-flake.flakeModules.default
        rust-flake.flakeModules.nixpkgs
      ];

      perSystem =
        { self', pkgs, ... }:
        {
          packages.default = self'.packages.tchat;

          devShells.default = pkgs.mkShell {
            inputsFrom = [ self'.devShells.rust ];
            packages = with pkgs; [
              nixd
              nixfmt-rfc-style
            ];
          };
        };
    };
}
