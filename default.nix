let
  oxalica = [ (import (builtins.fetchTarball "https://github.com/oxalica/rust-overlay/archive/master.tar.gz")) ];
  pkgs = (import (builtins.fetchTarball {
    url = "https://github.com/NixOS/nixpkgs/archive/4ecab3273592f27479a583fb6d975d4aba3486fe.zip";
    sha256 = "sha256:10wn0l08j9lgqcw8177nh2ljrnxdrpri7bp0g7nvrsn9rkawvlbf";
  }) { overlays = oxalica; });
  stdenv = pkgs.stdenv;
in pkgs.mkShell {
  buildInputs = [
    (pkgs.rust-bin.nightly."2023-01-01".minimal.override {
      targets = ["wasm32-unknown-unknown"];
    })
    pkgs.openssl
    pkgs.sqlite
    pkgs.wasm-pack
  ];
  nativeBuildInputs = [pkgs.pkg-config];
}
