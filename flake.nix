{
  description = "A development environment for Rust projects";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = {
    self,
    nixpkgs,
    flake-utils,
    rust-overlay,
  }:
    flake-utils.lib.eachDefaultSystem (
      system: let
        overlays = [(import rust-overlay)];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
        rustToolchain = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
      in {
        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            rustToolchain
            gcc
            pkg-config
            alsa-lib # libasound.so wanted by lib/libjsound.so
            fontconfig
            freetype
            stdenv.cc.cc # libstdc++.so.6
            xorg.libX11
            xorg.libXext
            xorg.libXi
            xorg.libXrender
            xorg.libXtst
            xorg.libXxf86vm
            xorg.libXt
            zlib
            cups.dev
          ];
        };

        packages.default = pkgs.rustPlatform.buildRustPackage {
          pname = "rust-jvm";
          src = ./.;
          cargoLock = {
            lockFile = ./Cargo.lock;
            allowBuiltinFetchGit = true;
          };
          cargoBuildFlags = [
            "--bin"
            "java"
          ];

          nativeBuildInputs = [

          ];
          buildInputs = [

          ];

          # If you have any runtime dependencies, add them here:
          # propagatedBuildInputs = [ ... ];
        };
      }
    );
}
