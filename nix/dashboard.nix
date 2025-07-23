{
  dockerTools,
  rustPlatform,
  openssl,
  pkg-config,
  runCommand,
  lib,
}:
let
  name = "dashboard";

  rustBin = rustPlatform.buildRustPackage {
    pname = name;
    version = "0.1.0";
    src = lib.cleanSourceWith {
      src = ../dashboard;
      filter =
        path: type:
        let
          relPath = lib.removePrefix (toString ../dashboard + "/") (toString path);
        in
        lib.any (p: lib.hasPrefix p relPath) [
          "src"
          "Cargo.toml"
          "Cargo.lock"
          "static"
          "templates"
        ];
    };

    cargoLock.lockFile = ../dashboard/Cargo.lock;

    nativeBuildInputs = [ pkg-config ];
    buildInputs = [ openssl ];
  };

  root = runCommand "root" { } ''
    mkdir -p $out/bin
    cp ${rustBin}/bin/${name} $out/bin/
  '';

in
dockerTools.buildImage {
  inherit name;
  tag = "latest";

  copyToRoot = root;

  config = {
    Cmd = [ "/bin/${name}" ];
    WorkingDir = "/";
    ExposedPorts = {
      "8000/tcp" = { };
    };
  };
}
