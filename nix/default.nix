{rustPlatform}:
rustPlatform.buildRustPackage {
  pname = "anyrun-ha-assist";
  version = "0.1.0";

  src = ./.;

  cargoLock.lockFile = ./Cargo.lock;
}
