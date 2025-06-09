{ pkgs ? import <nixpkgs> {} }:
let lib = pkgs.lib;
    agb-gbafix-src = pkgs.fetchFromGitHub {
      owner = "agbrs";
      repo = "agb";
      tag = "v0.21.3";
      hash = "sha256-V1uaVulFo+QZHcHbZIK4DfHMTun6duvK2K0nCnal1bs=";
    };
    agb-gbafix = with pkgs; rustPlatform.buildRustPackage (finalAttrs: {
      pname = "agb-gbafix";
      version = "0.21.3";
      name = "${finalAttrs.pname}-${finalAttrs.version}";

      src = agb-gbafix-src;

      buildAndTestSubdir = "agb-gbafix";

      cargoLock.lockFile = ./Cargo.lock;

      postPatch = ''
        ln -s ${./Cargo.lock} Cargo.lock
      '';
    });
in agb-gbafix
