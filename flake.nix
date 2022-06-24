{
  inputs = {
    flake-utils.url = "github:numtide/flake-utils";
    naersk.url = "github:nix-community/naersk";
    flake-compat = {
      url = "github:edolstra/flake-compat";
      flake = false;
    };
  };

  outputs = { self, nixpkgs, flake-utils, naersk, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages."${system}";
        naersk-lib = naersk.lib."${system}";
      in rec {
        # `nix build`
        packages.ckiesite = naersk-lib.buildPackage {
          pname = "ckiesite";
          root = ./.;
        };
        defaultPackage = packages.ckiesite;

        # `nix run`
        apps.ckiesite =
          flake-utils.lib.mkApp { drv = packages.ckiesite; };
        defaultApp = apps.ckiesite;

        # `nix develop`
        devShell =
          pkgs.mkShell { nativeBuildInputs = with pkgs; [ rustc cargo ]; };
      });
}
