{ pkgs }:
with pkgs;
mkShell
{
  buildInputs = [
    stdenv

    automake

    yarn
    yarn2nix
    nodePackages.typescript
    nodejs

    sops
  ];

  shellHook = ''
    PATH="$PWD/node_modules/.bin/:$PATH"
    # Mark variables which are modified or created for export.
    set -a
    source .env
    set +a
    exec $DEV_SHELL
  '';

}
