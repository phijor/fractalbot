{inputs, ...}: {
  config = {
    perSystem = {pkgs, ...}: let
      project = import ../Cargo.nix { inherit pkgs; };
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
