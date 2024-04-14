let
  pkgs = import <nixpkgs> {};
in
  pkgs.mkShell {
    buildInputs = with pkgs; [
        alsa-lib
        openssl
        xdotool
    ];
    nativeBuildInputs = with pkgs; [
        pkg-config
    ];
  }
