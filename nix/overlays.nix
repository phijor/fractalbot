{inputs,...}: {
  config = {
    perSystem = {
      inputs',
      system,
      ...
    }: let
      overlays = [
        (import inputs.rust-overlay)
        (self: super: let
          toolchain = super.rust-bin.fromRustupToolchainFile ../rust-toolchain.toml;
        in {
          rustc = toolchain;
        })
      ];
    in {
      _module.args.pkgs = import inputs.nixpkgs {
        inherit system overlays;
      };
    };
  };
}
