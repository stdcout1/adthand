{ pkgs ? import <nixpkgs> { } }:
with pkgs;
mkShell {
  buildInputs = [
    alsa-lib
    openssl
    xdotool
  ];
  nativeBuildInputs = [
    pkg-config
  ];
}
