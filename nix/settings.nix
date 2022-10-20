{ pkgs, env }:
let
  defaultConfig = rec {
    inherit env;
    url = "https://aurora.dev";
    user = "nobody";
    name = "rainbowbridge-watchdog";
    home = "bridge";
    pkgPrefix = "${name}-${env}";
  };
  environments = rec {
    development = { };
    production = { };
  };
in
defaultConfig // environments.${env}
