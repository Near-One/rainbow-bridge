{ pkgs
, app
}:
with pkgs;
let
  # https://github.com/NixOS/nixpkgs/blob/7a34bcc2a32493b204192cabb898f025cd4ba008/pkgs/build-support/docker/examples.nix#L374
  nonRootShadowSetup = { user ? app.user, uid ? 65534, gid ? uid }: with pkgs; [
    (
      writeTextDir "etc/shadow" ''
        root:!x:::::::
        ${user}:!:::::::
      ''
    )
    (
      writeTextDir "etc/passwd" ''
        root:x:0:0::/root:${runtimeShell}
        ${user}:x:${toString uid}:${toString gid}::/home/${user}:
      ''
    )
    (
      writeTextDir "etc/group" ''
        root:x:0:
        ${user}:x:${toString gid}:
      ''
    )
    (
      writeTextDir "etc/gshadow" ''
        root:x::
        ${user}:x::
      ''
    )
    (linkFarm "ssl" [{
      name = "etc/ssl/certs/ca-certificates.crt";
      path = "${cacert}/etc/ssl/certs/ca-bundle.crt";
    }])
  ];

  imagePrefix = "${app.name}-${app.env}";

  mkTmp = "mkdir -m 1777 tmp;";

  # Config file needed for container/host resolution.
  nsswitch-conf = writeTextFile {
    name = "nsswitch.conf";
    text = "hosts: files dns";
    destination = "/etc/nsswitch.conf";
  };

  defaultConfig = {
    User = app.user;
    WorkingDir = "/" + app.home;
    cmd = [ entrypoint ];
  };

  extraCommands = mkTmp + ''
    mkdir -p ${app.home}/{.rainbow,.pm2}
    chmod 777 ${app.home}/{.rainbow,.pm2}
    cd ${app.home}
  '';

  appImage = params:
    let
      buildArgs = {
        tag = "latest";
        config = {
          Env = [
            "SSL_CERT_FILE=${cacert}/etc/ssl/certs/ca-bundle.crt"
            "NODE_OPTIONS=--max-old-space-size=512"
            "HOME=/${app.home}"
            "NODE_PATH=/${app.home}/node_modules"
          ];
        };
      } // params;
    in
    pkgs.dockerTools.buildImage buildArgs;

  contents = [ pkgs.appPackage ];

in
rec {
  app-deps-image = appImage {
    name = "${imagePrefix}-app-deps";
    contents = with pkgs; [
      (nonRootShadowSetup { })
      app-deps
      cacert
      iana-etc
      nsswitch-conf
    ];
  };

  app-image = appImage {
    inherit contents extraCommands;
    name = "${imagePrefix}-app";
    fromImage = app-deps-image;
    config = defaultConfig;
  };

  app-image-stream = pkgs.dockerTools.streamLayeredImage {
    inherit contents extraCommands;
    name = "${imagePrefix}-app";
    tag = "latest";
    fromImage = app-deps-image;
    config = defaultConfig;
  };
}
