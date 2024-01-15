{
  description = "Description for the project";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

    flake-utils = {
      url = "github:numtide/flake-utils";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    nci = {
      url = "github:yusdacra/nix-cargo-integration";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    flake-parts = {
      url = "github:hercules-ci/flake-parts";
      inputs.nixpkgs-lib.follows = "nixpkgs";
    };
  };

  outputs =
    inputs @ { flake-parts
    , nci
    , flake-utils
    , ...
    }:
    flake-parts.lib.mkFlake
      {
        inherit inputs;
      }
      {
        systems = flake-utils.lib.defaultSystems;
        imports = [
          nci.flakeModule
          ./nix/fractalbot.nix
        ];


      };
}
