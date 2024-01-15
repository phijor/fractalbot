_: {
  perSystem =
    { pkgs
    , config
    , ...
    }:
    let
      crateName = "fractalbot";
      crateOutputs = config.nci.outputs.${crateName};
    in
    rec {
      nci.projects."simple".path = ./..;
      nci.crates.${crateName} = { };

      devShells.default = crateOutputs.devShell;
      packages.default = crateOutputs.packages.release;
      apps = {
        type = "app";
        program = "${packages.default}/bin/fractalbot";
      };
    };
}
