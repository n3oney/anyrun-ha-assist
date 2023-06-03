{
  rustPlatform,
  sqlite,
}:
rustPlatform.buildRustPackage {
  pname = "anyrun-ha-assist";
  version = "0.1.0";

  src = ../.;

  buildInputs = [
    sqlite
  ];

  cargoLock = {
    lockFile = ../Cargo.lock;
    outputHashes."anyrun-interface-0.1.0" = "sha256-tw4TCngDpP7ACt5HpEHFtxPxpNU8/o7DsQZBSPPSTA8=";
  };
}
