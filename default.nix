let
  oxalica = [ (import (builtins.fetchTarball "https://github.com/oxalica/rust-overlay/archive/master.tar.gz")) ];
  pkgs = (import (builtins.fetchTarball {
    url = "https://github.com/NixOS/nixpkgs/archive/a7ecde854aee5c4c7cd6177f54a99d2c1ff28a31.zip";
    sha256 = "162dywda2dvfj1248afxc45kcrg83appjd0nmdb541hl7rnncf02";
  }) { overlays = oxalica; });
  stdenv = pkgs.stdenv;
in pkgs.mkShell {
  buildInputs = [
    (pkgs.rust-bin.nightly."2023-01-01".minimal.override {
      targets = ["wasm32-unknown-unknown"];
    })
    pkgs.openssl
    pkgs.sqlite
  ];
  nativeBuildInputs = [pkgs.pkg-config];
  shellHook =
    ''
    cargo install --locked cargo-leptos
    '';
}
