rec {
  description = "Rainbow Bridge";

  inputs = {
    flake-utils.url = "github:numtide/flake-utils";
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-22.05";
  };

  outputs = inputs@{ self, nixpkgs, flake-utils, ... }:
    with flake-utils.lib; eachSystem allSystems (system:
      let
        pkgs = import nixpkgs { inherit system; };
        packages = pkgs.lib.foldr
          (a: b: a // b)
          { }
          (map
            (env: (pkgs.lib.mapAttrs'
              (pkgName: pkg: {
                name = "${env}-${pkgName}";
                value = pkg;
              })
              (
                let
                  settings = import ./nix/settings.nix { inherit pkgs env; };
                  app = settings;
                  pkgs = import nixpkgs {
                    inherit system app;
                    overlays = [
                      (import ./nix/environment.nix { inherit app; })
                    ];
                  };
                  docker = import ./nix/docker.nix { inherit pkgs app; };
                in
                rec {
                  image-stream = docker.app-image-stream;
                  image = docker.app-image;
                  devShell = import ./nix/shell.nix { inherit pkgs; };
                }
              ))
            )
            [ "production" "development" ]);
      in
      {
        packages = packages // {
          default = packages.development-image-stream;
        };
        devShells.default = packages.development-devShell;
      }
    );
}
