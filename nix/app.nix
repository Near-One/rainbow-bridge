{ pkgs, app }:
with pkgs;
let
  inherit (pkgs) lib;

  package = stdenv.mkDerivation rec {
    name = "${app.pkgPrefix}-app";

    src = lib.cleanSourceWith {
      filter = lib.cleanSourceFilter;
      src = lib.cleanSourceWith {
        filter = name: type:
          !(lib.hasSuffix ".nix" name);
        src = ../.;
      };
    };

    buildInputs = [ rsync nodejs ];

    # TODO move out NearBridge.json/other artefacts to a separate package
    # alternatively this could run a hardhat build but it does not work offline
    buildPhase = ''
      mkdir -p contracts/eth/nearbridge/artifacts/contracts/NearBridge.sol
      cp ./nix/NearBridge.json contracts/eth/nearbridge/artifacts/contracts/NearBridge.sol/NearBridge.json
      rsync -a ${yarnworkspace}/ ./
    '';

    installPhase = ''
      mkdir -p $out/{bin,${app.home}/.rainbow}
      rsync -a ./ $out/${app.home}/
      ln -s /${app.home}/cli/index.js $out/bin/rainbow-bridge
    '';

    meta = with lib; {
      homepage = app.url;
      description = "${app.name} (combined app)";
      platforms = platforms.all;
    };
  };
in
package
