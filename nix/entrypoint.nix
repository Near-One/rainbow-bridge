{ pkgs, app }:
with pkgs;
let
  inherit (pkgs) lib;

  entrypoint = writeScript "entrypoint.sh" ''
    #!/bin/sh
    set -e

    if [[ -z "''${SOPS_AGE_KEY}" ]]; then
      echo "SOPS_AGE_KEY is not specified, sops decryption will fail"
    fi

    if [[ -d secrets ]]; then
      mkdir -p /dev/shm/secrets/
      for i in $(ls secrets); do
        echo  "decrypting $i..."
        ${pkgs.sops}/bin/sops -d secrets/$i > /dev/shm/secrets/$i;
      done
      ln -s /dev/shm/secrets/config.json .rainbow/config.json
    else
      echo "$(pwd)/secrets directory not found, skipping sops decryption"
    fi

    rainbow-bridge start bridge-watchdog --daemon false
  '';
in
entrypoint
