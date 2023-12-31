{
  description = "Description for the project";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    rust-crate2nix.url = "github:nix-community/crate2nix";
  };

  outputs = inputs @ {
    flake-parts,
    rust-overlay,
    rust-crate2nix,
    ...
  }:
    flake-parts.lib.mkFlake {inherit inputs;} {
      systems = ["x86_64-linux" "aarch64-linux" "aarch64-darwin" "x86_64-darwin"];
      imports = [
        ./nix/overlays.nix
        ./nix/fractalbot.nix
      ];
      perSystem = {
        config,
        self',
        inputs',
        pkgs,
        system,
        ...
      }: {
        devShells.default = pkgs.mkShell {
          inputsFrom = [self'.packages.fractalbot];
        };
      };
    };
}
