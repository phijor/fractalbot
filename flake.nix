{
  description = "Description for the project";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-parts.url = "github:hercules-ci/flake-parts";
  };

  outputs =
    inputs@{ flake-parts, ... }:
    flake-parts.lib.mkFlake { inherit inputs; } {
      systems = [
        "x86_64-linux"
        "aarch64-linux"
        "aarch64-darwin"
        "x86_64-darwin"
      ];
      perSystem =
        { pkgs, ... }:
        {
          packages.default = pkgs.callPackage ./fractalbot.nix { };
          devShells.default = pkgs.mkShell {
            packages = with pkgs; [
              cargo
              cargo-outdated
              clippy
              rustc
              rust-analyzer
            ];

            RUST_BACKTRACE = 1;
          };
        };
    };
}
