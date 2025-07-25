{ rustPlatform, lib }:
let
  cargoToml = builtins.fromTOML (builtins.readFile ../node/Cargo.toml);
in
rustPlatform.buildRustPackage {
  pname = "moxapi";
  inherit (cargoToml.package) version;
  cargoLock.lockFile = ../node/Cargo.lock;

  src = ../node;

  postFixup = ''
    mkdir -p $out/share/systemd/user
    substitute $src/contrib/systemd/moxapi.service.in $out/share/systemd/user/moxapi.service --replace-fail '@bindir@' "$out/bin"
    chmod 0644 $out/share/systemd/user/moxapi.service
  '';

  meta = {
    description = "";
    mainProgram = "moxapi";
    homepage = "https://github.com/mox-desktop/moxapi";
    license = lib.licenses.mit;
    maintainers = builtins.attrValues { inherit (lib.maintainers) unixpariah; };
    platforms = lib.platforms.unix;
  };
}
