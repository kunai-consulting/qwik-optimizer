# This is a Nix configuration file. It is used to define the environment
# for the project. It is a declarative way to define the dependencies.
# It is used by the `nix develop` command to create a development environment
# with all the dependencies needed for the project.

# To update the dependencies, run `nix flake update`.
{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = { self, rust-overlay, nixpkgs }:
    let
      b = builtins;
      devShell = system: _pkgs:
        let
          overlays = [ (import rust-overlay) ];
          pkgs = import nixpkgs { inherit system overlays; };
        in {
          default = pkgs.mkShell {
            nativeBuildInputs = with pkgs; [
              bashInteractive
              gitMinimal

              # Provides rustc and cargo
              (rust-bin.stable.latest.default.override {
                # For rust-analyzer
                extensions = [ "rust-src" ];
                # For building wasm
                targets = [ "wasm32-unknown-unknown" ];
              })
            ];
          };
        };
    in { devShells = b.mapAttrs (devShell) nixpkgs.legacyPackages; };
}
