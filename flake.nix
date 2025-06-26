{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem
      (system:
        let
          pkgs = nixpkgs.legacyPackages.${system};
          nativeBuildInputs = [ pkgs.pkg-config ];
          buildInputs = with pkgs;[
            alsa-lib
            openssl
            xdotool
          ];
        in
        {
          devShells.default = pkgs.mkShell
            {
              inherit buildInputs nativeBuildInputs;
            };
          packages.default = pkgs.rustPlatform.buildRustPackage rec {
            inherit buildInputs nativeBuildInputs;
            pname = "adthand";
            version = "1";
            src = ./.;
            cargoBuildFlags = [ "--workspace" ];
            RUSTFLAGS = "-C target-cpu=x86-64";
            CARGO_PROFILE_RELEASE_LTO = "false";

            useFetchCargoVendor = true;
            cargoHash = "sha256-lsYvF0hdgOIp3UvVJt7GyaHnxoiQeWii+CuOf9fUpgI=";

            postFixup = ''
              mv $out/bin/dameon $out/bin/adthand-dameon
            '';
          };

        }
      );
}
