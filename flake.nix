{
  inputs = {
    flake-utils.url = "github:numtide/flake-utils";
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    naersk = {
      url = "github:nix-community/naersk";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    flake-compat = {
      url = "github:edolstra/flake-compat";
      flake = false;
    };
    mozillapkgs = {
      url = "github:mozilla/nixpkgs-mozilla";
      flake = false;
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, flake-utils, naersk, mozillapkgs, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages."${system}";
        mozilla = pkgs.callPackage (mozillapkgs + "/package-set.nix") { };
        rust = (mozilla.rustChannelOf {
          date = "2022-06-28"; # get the current date with `date -I`
          channel = "nightly";
          sha256 = "sha256-vHXJ3IqSOE55qSC0BMmCNJjgs4Bs0n0FhiKUf1yPoFQ";
        }).rust;
        naersk-lib = naersk.lib."${system}".override {
          cargo = rust;
          rustc = rust;
        };
      in rec {
        # `nix build`
        packages.ckiesite = naersk-lib.buildPackage {
          pname = "ckiesite";
          root = ./.;
        };
        defaultPackage = packages.ckiesite;

        # `nix run`
        apps.ckiesite = flake-utils.lib.mkApp { drv = packages.ckiesite; };
        defaultApp = apps.ckiesite;

        # `nix develop`
        devShell = pkgs.mkShell { nativeBuildInputs = with pkgs; [ rust ]; };
      });
}
