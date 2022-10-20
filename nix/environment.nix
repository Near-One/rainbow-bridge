{ app }:
self: super:
with super;

let
  yarnPreBuild = ''
    export HOME=$PWD/yarn_home
    yarn config --offline set yarn-offline-mirror ${offlineCache}/
    ${super.fixup_yarn_lock}/bin/fixup_yarn_lock yarn.lock
  '';
  packageJSON = ../package.json;
  yarnLock = ../yarn.lock;
  yarnNix = ./yarn.nix;
  offlineCache = super.yarn2nix-moretea.importOfflineCache yarnNix;
  yarnFlags = super.yarn2nix-moretea.defaultYarnFlags ++ [ "--verbose" "--no-progress" ];
in

rec {

  yarnworkspace = stdenv.mkDerivation
    {
      name = "rainbow-bridge-workspace";
      version = "1.0.0";

      src = lib.cleanSourceWith {
        filter = lib.cleanSourceFilter;
        src = lib.cleanSourceWith {
          filter = name: type:
            !(lib.hasSuffix ".nix" name);
          src = ../.;
        };
      };

      buildInputs = with nodePackages; [ yarn rsync git ];

      configurePhase = yarnPreBuild;

      buildPhase = ''
        yarn install ${lib.escapeShellArgs yarnFlags}
        rm -rf yarn_home
      '';

      installPhase = ''
        rsync -a ./ $out/
      '';

      passthru = {
        runtimeBinaries = [
          nodejs
          busybox
          sops
        ] ++ lib.optionals (app.env == "development") [
          fish
          bashInteractive
          git
          htop
          strace
          yarn
        ];
      };

    };

  appPackage = import ./app.nix { inherit app; pkgs = self; };
  entrypoint = import ./entrypoint.nix { inherit app; pkgs = self; };

  app-deps = buildEnv {
    name = "app-deps-env";
    paths = yarnworkspace.runtimeBinaries;
  };
}
