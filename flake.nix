{
  description = "chrio - Tauri physio photo manager";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-24.11";
    flake-utils.url = "github:numtide/flake-utils";
    fenix.url = "github:nix-community/fenix";
  };

  nixConfig = {
    trusted-community-caches = [
      "https://numtide.cachix.org"
      "https://nix-community.cachix.org"
    ];
  };

  outputs = { self, nixpkgs, flake-utils, fenix }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs { inherit system; };
      in
      {
        devShells.default = pkgs.mkShell {
          packages = [
            pkgs.bun
            pkgs.nodejs_20
            fenix.packages.${system}.stable.toolchain
            pkgs.clang
            pkgs.sqlite
            pkgs.pkg-config
            pkgs.libwebp
            pkgs.openssl
            pkgs.zlib
          ];
        };
      }
    );
}