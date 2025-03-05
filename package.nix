{
  rustPlatform,
  pkg-config,
  lib,
}:
rustPlatform.buildRustPackage {
  name = "new-project";
  src = ./.;

  cargoLock = {
    lockFile = ./Cargo.lock;
    allowBuiltinFetchGit = true;
  };

  nativeBuildInputs = [
    pkg-config
  ];
}
