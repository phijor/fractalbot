{
  lib,
  rustPlatform,
}:

rustPlatform.buildRustPackage (finalAttrs: {
  pname = "fractalbot";
  version = "0.3.3";
  src = lib.cleanSource ./.;
  cargoHash = "sha256-CEPWlFHpa57vsgQDL7yYLotGr/b5cpFfpp393EWxdqA=";
})
