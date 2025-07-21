{
  lib,
  rustPlatform,
}:

rustPlatform.buildRustPackage (finalAttrs: {
  pname = "fractalbot";
  version = "0.3.5";
  src = lib.cleanSourceWith {
    src = ./.;
    filter = path: type: !(type == "directory" && baseNameOf path == ".github");
  };
  cargoDeps = rustPlatform.importCargoLock {
    lockFile = ./Cargo.lock;
  };
})
