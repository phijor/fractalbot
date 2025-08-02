{
  lib,
  rustPlatform,
}:

let
  package = (lib.importTOML ./Cargo.toml).package;
in
rustPlatform.buildRustPackage (finalAttrs: {
  pname = package.name;
  version = package.version;
  src = lib.cleanSourceWith {
    src = ./.;
    filter = path: type: !(type == "directory" && baseNameOf path == ".github");
  };
  cargoDeps = rustPlatform.importCargoLock {
    lockFile = ./Cargo.lock;
  };
})
