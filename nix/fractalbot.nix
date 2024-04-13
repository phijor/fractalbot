{inputs, ...}: {
  config = {
    perSystem = {pkgs, ...}: let
      crateTools = pkgs.callPackage "${inputs.rust-crate2nix}/tools.nix" {inherit pkgs;};
      project = crateTools.appliedCargoNix {
        name = "fractalbot";
        src = ./..;
      };
      fractalbot = project.rootCrate.build;
      crate-dependencies = pkgs.symlinkJoin {
        name = "fractalbot-dependencies";
        paths = fractalbot.completeDeps;
      };
    in rec {
      packages = {
        inherit fractalbot crate-dependencies;
        default = fractalbot;
      };
      apps = {
        type = "app";
        program = "${fractalbot}/bin/fractalbot";
      };
    };
  };
}
