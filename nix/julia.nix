{inputs, ...}: {
  config = {
    perSystem = {pkgs, ...}: let
      crateTools = pkgs.callPackage "${inputs.rust-crate2nix}/tools.nix" {inherit pkgs;};
      cargoNix = crateTools.generatedCargoNix {
        name = "julia";
        src = ./..;
      };
      project = import cargoNix {inherit pkgs;};
      julia = project.rootCrate.build;
    in rec {
      packages = {
        inherit julia;
        default = julia;
      };
      # apps = {
      #   type = "app";
      #   program = "${julia}/bin/julia";
      # };
    };
  };
}
