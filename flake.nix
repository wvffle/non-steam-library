{
  description = "Non Steam Library Manager";

  inputs = {
    nixpkgs.url = github:NixOS/nixpkgs/nixos-unstable;

    flake-utils.url = "github:numtide/flake-utils";
    flake-utils.inputs.nixpkgs.follows = "nixpkgs";

    rust-overlay.url = "github:oxalica/rust-overlay";
    rust-overlay.inputs.nixpkgs.follows = "nixpkgs";
  };

  outputs = inputs: with inputs; flake-utils.lib.eachDefaultSystem (system: let
    pkgs = import nixpkgs {
      inherit system;
      overlays = [
        rust-overlay.overlays.default
      ];
    };

    rusttoolchain = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
    cargoToml = (builtins.fromTOML (builtins.readFile ./Cargo.toml));
    sharedDeps = with pkgs; [ rusttoolchain pkg-config ];
  in {
    packages = rec {
      non-steam-library = pkgs.rustPlatform.buildRustPackage {
        pname = cargoToml.package.name;
        version = cargoToml.package.version;
        src = ./.;
        cargoLock = {
          outputHashes = {};
          lockFile = ./Cargo.lock;
        };

        nativeBuildInputs = sharedDeps;
      };
          
      default = non-steam-library;
    };


    devShell = pkgs.mkShell {
      buildInputs = sharedDeps;
    };
  });
}
