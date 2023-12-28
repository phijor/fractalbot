{inputs, ...}: {
  config = {
    perSystem = {pkgs, ...}: let
      crateTools = pkgs.callPackage "${inputs.rust-crate2nix}/tools.nix" {inherit pkgs;};
      project = crateTools.appliedCargoNix {
        name = "fractalbot";
        src = ./..;
      };
      fractalbot = project.rootCrate.build;
    in rec {
      packages = {
        inherit fractalbot;
        default = fractalbot;
      };
      apps = {
        type = "app";
        program = "${fractalbot}/bin/fractalbot";
      };
    };
  };
}
