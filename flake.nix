{
  description = "An anyrun plugin that lets you use Home Assistant's Assist.";

  inputs = {
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = {
    nixpkgs,
    flake-utils,
    ...
  }:
    flake-utils.lib.eachDefaultSystem (
      system: let
        pkgs = nixpkgs.legacyPackages.${system};
      in rec {
        devShell = with pkgs;
          mkShell {
            buildInputs = [
              cargo
              rustc
              git
            ];
          };

        packages = rec {
          anyrun-ha-assist = pkgs.callPackage ./nix {};
          default = anyrun-ha-assist;
        };

        legacyPackages = packages;
      }
    );
}
